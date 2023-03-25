macro_rules! append_to_history {
    ($pool: ident, $body: ident, $response: ident) => {
        append_history(
            &$pool.clone(),
            $body.user.clone(),
            openchad_common::chat::ChatMessage {
                role: "user".into(),
                content: $body.message,
            },
        )
        .await
        .unwrap();
    };
}

use std::collections::HashMap;
use std::env::var;
use std::ops::{Add, Sub};
use std::sync::Arc;

use architectury::coreutils::cat;
use architectury::prelude::*;
use async_recursion::async_recursion;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Extension, Json, Router};
use axum_streams::StreamBodyAs;
use chrono::{Duration, FixedOffset, Local, TimeZone};
use eyre::Context;
use eyre::{eyre, ErrReport};
use futures::{pin_mut, Stream, StreamExt, TryStreamExt};
use minijinja::Environment;
use openchad_common::botconfig::{
    BotConfig, BotConfigHeadless, ConfigMacro, ConfigResponse, ConfigSearchQuery, Transform,
};
use openchad_common::chat::ChatMessage;
use openchad_common::search::SearchResponse;
use openchad_common::{CategorizeBody, CategorizeResponse, ChatBody};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Row, SqlitePool};

use crate::search::search;
use crate::{chat, get_full_history, internal_error_string};

pub fn read_config() -> Result<BotConfig> {
    Ok(serde_json::from_str(&cat(var("CONFIG_PATH")?)?)?)
}

pub fn create_routes(
    router: Router,
    config: &'static Arc<BotConfig>,
    config_json: Arc<Value>,
) -> Result<Router> {
    let help_config = config.clone();
    let categorize_config: BotConfigHeadless = config.clone().into();

    let mut env = Environment::new();
    let prompt = help_config.help_prompt.join("\n");
    env.add_template("template", &prompt)?;
    let rendered = env
        .get_template("template")?
        .render(categorize_config.clone())
        .unwrap();

    let mut env = Environment::new();
    let prompt = help_config.categorize_prompt.join("\n");
    env.add_template("template", &prompt)?;

    let categorized_rendered = env
        .get_template("template")?
        .render(categorize_config)
        .unwrap();

    Ok(config
        .endpoints
        .iter()
        .fold(Ok(router), |router: Result<Router>, (url, endpoint)| {
            let config = config.clone();
            let config_json = config_json.clone();

            let v = endpoint.task.split('.').collect::<Vec<_>>();

            if !config_json[v[0]][v[1]].is_object()
                || !(endpoint.task.starts_with("responses.")
                    || endpoint.task.starts_with("macros."))
            {
                return Err::<Router, ErrReport>(eyre!(
                    "Invalid task id `{}` for route {}",
                    endpoint.task,
                    url,
                ));
            }
            Ok(router?.route(
                &url,
                get(
                    async move |Extension(pool): Extension<SqlitePool>,
                                Json(body): Json<ChatBody>|
                                -> Result<StreamBodyAs, (StatusCode, String)> {
                        let history = get_full_history(&pool, &body.user).await?;

                        let response = resolve_task_stream(
                            endpoint.task.clone(),
                            config,
                            config_json,
                            Transform::new(),
                            body.message.clone(),
                            history,
                        )
                        .await
                        .map_err(|e| internal_error_string(e))?;

                        append_to_history!(pool, body, response);

                        Ok(StreamBodyAs::json_nl(response.filter_map(|i| async move {
                            if let Ok(i) = i {
                                Some(i)
                            } else {
                                None
                            }
                        })))
                    },
                ),
            ))
        })?
        .route(
            "/chat/help",
            get(
                async move |Extension(pool): Extension<SqlitePool>,
                            Json(body): Json<ChatBody>|
                            -> Result<StreamBodyAs, (StatusCode, String)> {
                    let history = get_full_history(&pool, &body.user).await?;

                    let response = chat::chat_request(&rendered, body.message.clone(), &history, config.clone())
                        .await
                        .unwrap();

                    append_to_history!(pool, body, response);

                    Ok(StreamBodyAs::json_nl(response.filter_map(|i| async move {
                        if let Ok(i) = i {
                            Some(i)
                        } else {
                            None
                        }
                    })))
                },
            ),
        )
        .route(
            "/categorize",
            get(
                async move |Extension(pool): Extension<SqlitePool>,
                            Json(body): Json<CategorizeBody>|
                            -> Result<Json<CategorizeResponse>, (StatusCode, String)> {
                    let history: Vec<ChatMessage> = sqlx::query(include_str!("../sql/ChatHistoryCategorize.sql"))
                        .bind(body.user)
                        .fetch_all(&pool)
                        .await
                        .context("Failed to query history").unwrap()
                        //.map_err(internal_error_string)?
                        .into_iter()
                        .map(|h| ChatMessage {
                            role: h.get("role"),
                            content: h.get("message"),
                        })
                        .collect();

                    let res = chat::chat_request(
                        &categorized_rendered,
                        body.message.clone(),
                        history
                                .get(history.len().saturating_sub(2)..)
                                .unwrap_or_default(),
                                config.clone()
                        )
                        .await.map_err(internal_error_string)?
                        .map_ok(|s| s)
                        .try_collect::<Vec<String>>()
                        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                        .concat();

                    info!("<Categorize> Resolved category: {}", res);

                    if help_config.endpoints.iter().fold(false, |acc, (_, c)| acc || res == c.id) {
                        Ok(Json(CategorizeResponse {
                            category: res,
                        }))
                    } else {
                        Ok(Json(CategorizeResponse {
                            category: help_config.fallback_endpoint.clone()
                        }))
                    }
                }
            )
        )
    )
}

#[derive(Serialize, Clone)]
struct ResponseContext {
    input: String,
    datetime: String,
    props: HashMap<String, String>,
    transform: Transform,
}

#[derive(Serialize)]
struct TransformContext {
    #[serde(flatten)]
    response_context: ResponseContext,
    output: String,
}

#[derive(Serialize)]
struct SearchContext {
    #[serde(flatten)]
    response_context: ResponseContext,
    response: SearchResponse,
}

#[derive(Serialize)]
struct SearchTransformContext {
    #[serde(flatten)]
    search_context: SearchContext,
    output: String,
}

#[async_recursion]
async fn resolve_task_stream(
    task: String,
    config: Arc<BotConfig>,
    config_json: Arc<Value>,
    transform: Transform,
    input: String,
    history: Vec<ChatMessage>,
) -> Result<impl Stream<Item = Result<String, std::io::Error>>> {
    if task.starts_with("responses.") {
        let v = task.split('.').collect::<Vec<_>>();
        let response_config: ConfigResponse =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        info!("<{task}> Resolving as stream");

        let mut env = Environment::new();
        let prompt = response_config.prompt.join("\n");
        env.add_template("template", &prompt)?;

        let response_context = ResponseContext {
            input: input.clone(),
            datetime: datetime(),
            props: config.props.clone(),
            transform,
        };

        let rendered = env
            .get_template("template")?
            .render(&response_context)
            .unwrap();

        let response = chat::chat_request(
            &rendered,
            if let Some(input_header) = response_config.input {
                let mut env = Environment::new();
                env.add_template("template", &input_header)?;
                env.get_template("template")?
                    .render(&response_context)
                    .unwrap()
            } else {
                input
            },
            &history,
            config,
        )
        .await
        .unwrap();

        let footer = response_config.footer.unwrap_or_default();

        let mut env = Environment::new();
        env.add_template("template", &footer)?;
        let rendered = env
            .get_template("template")?
            .render(response_context)
            .unwrap();

        return Ok(async_stream::stream! {
            let mut acc = String::new();

            pin_mut!(response);

            while let Some(Ok(part)) = response.next().await {
                acc += &part;

                let ret: Result<String, std::io::Error> = Ok(part.clone());

                yield ret;
            }

            yield Ok(rendered);
        });
    } else if task.starts_with("macros.") {
        let v = task.split('.').collect::<Vec<_>>();
        let macro_config: ConfigMacro = serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        let mut transform = transform;
        let mut input = input;

        for (i, i_task) in macro_config.tasks.iter().enumerate() {
            if i == macro_config.tasks.len() - 1 {
                break;
            }

            info!(
                "<{task}> Resolving task `{}` ({}/{})",
                i_task,
                i + 1,
                macro_config.tasks.len()
            );

            // dont worry abt it :)
            (input, transform) = resolve_task(
                i_task.clone(),
                config.clone(),
                config_json.clone(),
                transform.clone(),
                input.clone(),
                history.clone(),
            )
            .await?;
        }

        info!(
            "<{task}> Resolving streaming task `{}`",
            macro_config.tasks.last().unwrap()
        );

        return resolve_task_stream(
            macro_config.tasks.last().unwrap().clone(),
            config,
            config_json,
            transform,
            input,
            history,
        )
        .await;
    } else {
        info!("<{task}> Streaming error");
        return Err(eyre!("`{}` isn't a member of `responses` or `macros`. It can't be resolved to a stream and presented to the user.", task));
    }
}

async fn resolve_task(
    task: String,
    config: Arc<BotConfig>,
    config_json: Arc<Value>,
    transform: Transform,
    input: String,
    history: Vec<ChatMessage>,
) -> Result<(String, Transform)> {
    if task.starts_with("responses.") {
        let v = task.split('.').collect::<Vec<_>>();
        let response_config: ConfigResponse =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        let mut env = Environment::new();
        let prompt = response_config.prompt.join("\n");
        env.add_template("template", &prompt)?;

        let response_context = ResponseContext {
            input: input.clone(),
            datetime: datetime(),
            props: config.props.clone(),
            transform,
        };

        let rendered = env
            .get_template("template")?
            .render(&response_context)
            .unwrap();

        let reponse = chat::chat_request(
            &rendered,
            if let Some(input_header) = response_config.input {
                let mut env = Environment::new();
                env.add_template("template", &input_header)?;
                env.get_template("template")?
                    .render(&response_context)
                    .unwrap()
            } else {
                input
            },
            &history,
            config,
        )
        .await?
        .map_ok(|s| s)
        .try_collect::<Vec<String>>()
        .await?
        .concat();

        let mut transform = Transform::new();

        if let Some(pre_transform) = &response_config.transform {
            let transform_context = TransformContext {
                response_context,
                output: reponse.clone(),
            };

            for (k, v) in pre_transform {
                let mut env = Environment::new();
                env.add_template("template", v)?;
                let rendered = env
                    .get_template("template")?
                    .render(&transform_context)
                    .unwrap();

                transform.insert(k.clone(), rendered);
            }
        }

        Ok((reponse, transform))
    } else if task.starts_with("searchQueries.") {
        let v = task.split('.').collect::<Vec<_>>();
        let search_config: ConfigSearchQuery =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        let response_context = ResponseContext {
            input: input.clone(),
            datetime: datetime(),
            props: config.props.clone(),
            transform,
        };

        let search_response = search(
            if let Some(input_header) = search_config.input {
                let mut env = Environment::new();
                env.add_template("template", &input_header)?;
                env.get_template("template")?
                    .render(&response_context)
                    .unwrap()
            } else {
                input
            },
            serde_json::from_value(
                serde_json::to_value(search_config.query_params)?
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .map(|(k, v)| {
                        if let Some(v) =
                            serde_json::from_value::<Option<String>>(v.clone()).unwrap()
                        {
                            let mut env = Environment::new();
                            env.add_template("template", &v).unwrap();
                            let rendered = env
                                .get_template("template")
                                .unwrap()
                                .render(&response_context)
                                .unwrap();

                            (k.clone(), Value::String(rendered))
                        } else {
                            (k.clone(), v.clone())
                        }
                    })
                    .collect::<Value>(),
            )?,
        )
        .await?;

        let mut env = Environment::new();
        env.add_template("template", &search_config.output)?;

        let search_context = SearchContext {
            response_context: response_context.clone(),
            response: search_response,
        };

        let rendered = env.get_template("template")?.render(&search_context)?;

        let mut transform = Transform::new();

        if let Some(pre_transform) = &search_config.transform {
            let search_transform_context = SearchTransformContext {
                search_context,
                output: rendered.clone(),
            };

            for (k, v) in pre_transform {
                let mut env = Environment::new();
                env.add_template("template", v)?;
                let rendered = env
                    .get_template("template")?
                    .render(&search_transform_context)
                    .unwrap();

                transform.insert(k.clone(), rendered);
            }
        }

        Ok((rendered, transform))
    } else {
        Err(eyre!("`{}` isn't a member of `responses` or `searchQueries`. It can't be run as an intermediate task.", task))
    }
}

async fn append_history(
    pool: &SqlitePool,
    username: String,
    message: ChatMessage,
) -> Result<(), (StatusCode, String)> {
    sqlx::query(
        r#"insert into ChatHistory (username, message, role)
    values ($1, $2, $3)"#,
    )
    .bind(username)
    .bind(message.content)
    .bind(message.role)
    .execute(pool)
    .await
    .context("Failed to append history")
    .map_err(internal_error_string)?;

    Ok(())
}

fn datetime() -> String {
    let est = FixedOffset::west_opt(180).unwrap();
    let dt = est
        .from_utc_datetime(&Local::now().naive_utc())
        .sub(Duration::days(1))
        .add(Duration::hours(12));

    dt.format("%Y-%m-%d %I:%M %p").to_string()
}
