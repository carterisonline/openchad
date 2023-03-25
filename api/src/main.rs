#![feature(async_closure)]

mod botconfig;
mod chat;
mod search;

use std::env::var;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use architectury::log::Report;
use architectury::prelude::*;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Extension, Json, Router};
use eyre::Context;
use once_cell::sync::Lazy;
use openchad_schemas::botconfig::BotConfig;
use openchad_schemas::chat::ChatMessage;
use openchad_schemas::HistoryBody;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::sqlite::{SqliteJournalMode, SqlitePool};
use sqlx::{ConnectOptions, Connection, Pool, Row, Sqlite};

use crate::botconfig::{create_routes, read_config};

static CONFIG: Lazy<Arc<BotConfig>> = Lazy::new(|| Arc::new(read_config().unwrap()));

async fn init_pool() -> Result<Pool<Sqlite>> {
    let sqlite_url = var("DATABASE_URL")?;

    let conn = SqliteConnectOptions::from_str(&sqlite_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true)
        .connect()
        .await?;

    conn.close();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&sqlite_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

#[tokio::main]

async fn main() -> Result<()> {
    architectury::init();

    let pool = init_pool().await?;

    let config = Arc::new(read_config()?);
    let config_json = Arc::new(serde_json::to_value(config.as_ref())?);

    let app = create_routes(
        Router::new().route("/history", post(history)),
        &CONFIG,
        config_json.clone(),
    )?
    .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 1381));
    debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn history(
    Extension(pool): Extension<SqlitePool>,
    Json(body): Json<HistoryBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    append_history(
        &pool,
        body.user,
        ChatMessage {
            role: "assistant".into(),
            content: body.message,
        },
    )
    .await?;

    Ok(StatusCode::OK)
}

fn internal_error_string(err: Report) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn get_full_history(
    pool: &SqlitePool,
    user: &str,
) -> Result<Vec<ChatMessage>, (StatusCode, String)> {
    Ok(sqlx::query(include_str!("../sql/ChatHistoryFull.sql"))
        .bind(user)
        .fetch_all(pool)
        .await
        .context("Failed to query history")
        .map_err(internal_error_string)?
        .into_iter()
        .map(|h| ChatMessage {
            role: h.get("role"),
            content: h.get("message"),
        })
        .collect())
}

async fn append_history(
    pool: &SqlitePool,
    username: String,
    message: ChatMessage,
) -> Result<(), (StatusCode, String)> {
    sqlx::query(include_str!("../sql/ChatHistoryInsert.sql"))
        .bind(username)
        .bind(message.content)
        .bind(message.role)
        .execute(pool)
        .await
        .context("Failed to append history")
        .map_err(internal_error_string)?;

    Ok(())
}
