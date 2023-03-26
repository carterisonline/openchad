macro_rules! append_to_history {
    ($pool: ident, $body: ident, $response: ident) => {
        append_history(
            &$pool.clone(),
            $body.user.clone(),
            openchad_schemas::chat::ChatMessage {
                role: "user".into(),
                content: $body.message,
            },
        )
        .await
        .unwrap();
    };
}

macro_rules! merge_request_parts {
    ($t: ident, $rt: ident, $rp: expr, $pd: ident, $ctx: ident) => {
        $rp.iter()
            .filter(|(_, prop_options)| prop_options.redirect == Redirect::$rt)
            .map(|(prop, prop_options)| (prop.clone(), prop_options.value.clone()))
            .chain($pd.$t.unwrap_or_default().into_iter().map(|(k, v)| {
                let mut env = Environment::new();
                env.add_template("template", &v).unwrap();
                let rendered = env.get_template("template").unwrap().render(&$ctx).unwrap();
                (k, rendered)
            }))
            .collect::<HashMap<_, _>>()
    };
}

macro_rules! json_nl_stream {
    ($response: ident) => {
        Ok(axum_streams::StreamBodyAs::json_nl($response.filter_map(
            |i| async move {
                if let Ok(i) = i {
                    Some(i)
                } else {
                    None
                }
            },
        )))
    };
}

use std::collections::{HashMap, HashSet};
use std::env::{self, var};
use std::ops::{Add, Sub};
use std::path::PathBuf;
use std::sync::Arc;

use architectury::coreutils::cat;
use architectury::prelude::*;
use async_recursion::async_recursion;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::routing::get;
use axum::{Extension, Json, Router};
use axum_streams::StreamBodyAs;
use chrono::{Duration, FixedOffset, Local, TimeZone};
use eyre::{eyre, ErrReport};
use eyre::{Context, ContextCompat};
use futures::{pin_mut, Stream, StreamExt, TryStreamExt};
use minijinja::Environment;
use once_cell::sync::Lazy;
use openchad_schemas::botconfig::{
    BotConfig, BotConfigHeadless, ConfigMacro, ConfigProvider, ConfigResponse, Transform,
};
use openchad_schemas::chat::ChatMessage;
use openchad_schemas::provider::{Provider, Redirect};
use openchad_schemas::{CategorizeBody, CategorizeResponse, ChatBody};
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;

use crate::{chat, get_history, internal_error_string};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ResponseContext {
    input: String,
    datetime: String,
    props: HashMap<String, String>,
    transform: Transform,
    args: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ResponseContextWithOutput {
    #[serde(flatten)]
    response_context: ResponseContext,
    output: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderContext {
    #[serde(flatten)]
    response_context: ResponseContext,
    env: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProviderTransformContext {
    #[serde(flatten)]
    response_context: ProviderContext,
    response: Value,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MacroStartContext {
    input: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MacroContext {
    #[serde(flatten)]
    response_context: ResponseContext,
    #[serde(rename = "macro")]
    macro_: MacroStartContext,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderPropOptions {
    required: bool,
    redirect: Redirect,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderPropOptionsOnly {
    required: bool,
    redirect: Redirect,
}

static ENV: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_function("trim", trim);
    env
});

fn trim(source: &str, matches: &str) -> String {
    source
        .trim_matches(matches.chars().next().unwrap())
        .to_string()
}

fn template<T: Serialize, S: AsRef<str>>(source: S, context: &T) -> Result<String> {
    let mut env = ENV.clone();
    env.add_template("template", source.as_ref())?;
    Ok(env.get_template("template")?.render(context).unwrap())
}

fn template_multiline<T: Serialize>(source: &Vec<String>, context: &T) -> Result<String> {
    template(source.join("\n"), context)
}

pub fn read_config() -> Result<BotConfig> {
    Ok(serde_json::from_str(&cat(var("CONFIG_PATH")?)?)?)
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

pub fn create_routes(
    router: Router,
    config: &'static Arc<BotConfig>,
    config_json: Arc<Value>,
) -> Result<Router> {
    let gen_config = config.clone();
    let categorize_config: BotConfigHeadless = config.clone().into();

    let help_prompt = template_multiline(&gen_config.help_prompt, &categorize_config)?;
    let categorize_prompt = template_multiline(&gen_config.categorize_prompt, &categorize_config)?;

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
                        let history = get_history(include_str!("../sql/ChatHistoryFull.sql"), &pool, &body.user).await?;

                        let response = resolve_task_stream(
                            endpoint.task.clone(),
                            config,
                            config_json,
                            Transform::new(),
                            body.message.clone(),
                            history,
                            HashMap::new(),
                        )
                        .await
                        .map_err(internal_error_string)?;

                        append_to_history!(pool, body, response);
                        json_nl_stream!(response)
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
                    let history = get_history(include_str!("../sql/ChatHistoryFull.sql"), &pool, &body.user).await?;

                    let response = chat::chat_request(&help_prompt, body.message.clone(), &history, config.clone())
                        .await
                        .unwrap();

                    append_to_history!(pool, body, response);
                    json_nl_stream!(response)
                },
            ),
        )
        .route(
            "/categorize",
            get(
                async move |Extension(pool): Extension<SqlitePool>,
                            Json(body): Json<CategorizeBody>|
                            -> Result<Json<CategorizeResponse>, (StatusCode, String)> {
                    let history = get_history(include_str!("../sql/ChatHistoryCategorize.sql"), &pool, &body.user).await?;

                    let category = chat::chat_request(
                        &categorize_prompt,
                        body.message.clone(),
                        history
                                .get(history.len().saturating_sub(2)..)
                                .unwrap_or_default(),
                                config.clone()
                        )
                        .await.map_err(internal_error_string)?
                        .try_collect::<Vec<String>>()
                        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                        .concat();

                    info!("<Categorize> Resolved category: {category}");

                    if gen_config.endpoints.iter().any(|(_, c)| category == c.id) {
                        Ok(Json(CategorizeResponse {
                            category,
                        }))
                    } else {
                        Ok(Json(CategorizeResponse {
                            category: gen_config.fallback_endpoint.clone()
                        }))
                    }
                }
            )
        )
    )
}

#[async_recursion]
async fn resolve_task_stream(
    task: String,
    config: Arc<BotConfig>,
    config_json: Arc<Value>,
    transform: Transform,
    input: String,
    history: Vec<ChatMessage>,
    args: HashMap<String, String>,
) -> Result<impl Stream<Item = Result<String, std::io::Error>>> {
    if task.starts_with("responses.") {
        let v = task.split('.').collect::<Vec<_>>();
        let response_config: ConfigResponse =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        info!("<{task}> Resolving as stream");

        let response_context = ResponseContext {
            input: input.clone(),
            datetime: datetime(),
            props: config.props.clone(),
            transform,
            args: args.clone(),
        };

        let prompt = template_multiline(&response_config.prompt, &response_context)?;

        let response = chat::chat_request(
            &prompt,
            args.get("input").unwrap_or(&input).clone(),
            &history,
            config,
        )
        .await
        .unwrap();

        let footer = template(
            response_config.footer.unwrap_or_default(),
            &response_context,
        )?;

        return Ok(async_stream::stream! {
            let mut acc = String::new();

            pin_mut!(response);

            while let Some(Ok(part)) = response.next().await {
                acc += &part;

                let ret: Result<String, std::io::Error> = Ok(part.clone());

                yield ret;
            }

            yield Ok(footer);
        });
    } else if task.starts_with("macros.") {
        let v = task.split('.').collect::<Vec<_>>();
        let macro_config: ConfigMacro = config.macros.get(v[1]).unwrap().clone();

        let macro_start_context = MacroStartContext {
            input: input.clone(),
        };

        let mut transform = transform;
        let mut input = input;
        let mut input_args = HashMap::new();

        for (i, (inst, inst_args)) in macro_config.iter().enumerate() {
            let context = MacroContext {
                response_context: ResponseContext {
                    input: input.clone(),
                    datetime: datetime(),
                    props: config.props.clone(),
                    transform: transform.clone(),
                    args: args.clone(),
                },
                macro_: macro_start_context.clone(),
            };

            input_args = inst_args
                .into_iter()
                .map(|(k, v)| (k.clone(), template(v, &context).unwrap()))
                .collect();

            input = input_args.get("input").unwrap_or(&input).clone();

            if i == macro_config.len() - 1 {
                break;
            }

            info!(
                "<{task}> ({}/{}) Resolving task `{}`",
                i + 1,
                macro_config.len(),
                inst
            );

            // dont worry abt it :)
            (input, transform) = resolve_task(
                inst.clone(),
                config.clone(),
                config_json.clone(),
                transform.clone(),
                input.clone(),
                history.clone(),
                input_args.clone(),
            )
            .await?;
        }

        info!(
            "<{task}> ({c}/{c}) Resolving streaming task `{t}`",
            c = macro_config.len(),
            t = macro_config.iter().last().unwrap().0
        );

        return resolve_task_stream(
            macro_config.iter().last().unwrap().0.clone(),
            config,
            config_json,
            transform,
            input,
            history,
            input_args,
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
    mut transform: Transform,
    input: String,
    history: Vec<ChatMessage>,
    args: HashMap<String, String>,
) -> Result<(String, Transform)> {
    if task.starts_with("responses.") {
        let v = task.split('.').collect::<Vec<_>>();
        let response_config: ConfigResponse =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        let context = ResponseContext {
            input: input.clone(),
            datetime: datetime(),
            props: config.props.clone(),
            transform: transform.clone(),
            args: args.clone(),
        };

        let prompt = template_multiline(&response_config.prompt, &context)?;

        let reponse = chat::chat_request(
            &prompt,
            args.get("input").unwrap_or(&input).clone(),
            &history,
            config,
        )
        .await?
        .try_collect::<Vec<String>>()
        .await?
        .concat();

        let response_context_with_output = ResponseContextWithOutput {
            response_context: context,
            output: reponse.clone(),
        };

        let footer = template(
            response_config.footer.unwrap_or_default(),
            &response_context_with_output,
        )?;

        response_config
            .transform
            .unwrap_or_default()
            .into_iter()
            .for_each(|(k, v)| {
                transform.insert(
                    k.clone(),
                    template(v, &response_context_with_output).unwrap(),
                );
            });

        Ok((reponse + &footer, transform))
    } else if task.starts_with("providers.") {
        let v = task.split('.').collect::<Vec<_>>();
        let provider_config: ConfigProvider =
            serde_json::from_value(config_json[v[0]][v[1]].clone())?;

        let provider_def: Provider = serde_json::from_str(
            &cat(PathBuf::from(var("PROVIDERS_PATH").unwrap())
                .join(format!("{}.json", provider_config.provider)))
            .context(format!("{:?} is not a provider", provider_config.provider))?,
        )
        .context(format!(
            "{:?} is an invalid provider",
            provider_config.provider
        ))?;

        info!(
            "<{task}> Resolving with provider `{provider}`",
            provider = provider_config.provider
        );

        let context = ProviderContext {
            response_context: ResponseContext {
                input: input.clone(),
                datetime: datetime(),
                props: config.props.clone(),
                transform: transform.clone(),
                args,
            },
            env: env::vars()
                .filter(|(k, _)| provider_def.env.clone().unwrap_or_default().contains(k))
                .collect(),
        };

        let url = template(&provider_def.url, &context)?;

        let props: HashMap<String, String> = provider_config
            .props
            .into_iter()
            .map(|(k, v)| (k, template(&v, &context).unwrap()))
            .collect();

        let resolved_props: HashMap<String, ProviderPropOptions> = props
            .into_iter()
            .map(|(prop, prop_val)| {
                let rules = provider_def
                    .prop_rules
                    .iter()
                    .find(|rule| rule.props.contains(&prop))
                    .context(format!("No prop rule found for {prop:?}"))
                    .unwrap();

                (
                    prop,
                    ProviderPropOptions {
                        required: rules.required,
                        redirect: rules.redirect.clone(),
                        value: prop_val,
                    },
                )
            })
            .collect();

        let all_required = provider_def
            .prop_rules
            .iter()
            .filter(|rule| rule.required)
            .map(|rule| rule.props.clone())
            .flatten()
            .collect::<HashSet<_>>();

        all_required.iter().for_each(|required_prop| {
            if !resolved_props
                .iter()
                .filter(|(_, prop_options)| prop_options.required)
                .any(|(prop, _)| prop == required_prop)
            {
                panic!("Required prop `{}` not found", required_prop);
            }
        });

        resolved_props
            .iter()
            .filter(|(_, prop_options)| prop_options.required)
            .for_each(|(k, v)| {
                info!(
                    "<{task}> Required prop {k:?} = `{v}`",
                    v = template(&v.value, &context).unwrap()
                );
            });

        let query = merge_request_parts!(query, Query, resolved_props, provider_def, context);
        let body = merge_request_parts!(body, Body, resolved_props, provider_def, context);
        let headers = merge_request_parts!(headers, Headers, resolved_props, provider_def, context);

        info!("<{task}> GET {url:?}");

        let response: Value = reqwest::Client::new()
            .get(url)
            .json(&body)
            .query(&query)
            .headers(HeaderMap::from_iter(headers.iter().map(|(k, v)| {
                (
                    HeaderName::from_bytes(k.as_bytes()).unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                )
            })))
            .send()
            .await?
            .json()
            .await?;

        let transform_context = ProviderTransformContext {
            response_context: context.clone(),
            response,
        };

        provider_config.transform.into_iter().for_each(|(k, v)| {
            transform.insert(k.clone(), template(v, &transform_context).unwrap());
        });

        Ok((String::new(), transform))
    } else {
        Err(eyre!("`{}` isn't a member of `responses` or `searchQueries`. It can't be run as an intermediate task.", task))
    }
}
