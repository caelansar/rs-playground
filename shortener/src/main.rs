mod error;

use crate::error::Error;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::pool::PoolOptions;
use sqlx::Error::Database;
use sqlx::{FromRow, PgPool};
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

const LISTEN_ADDR: &str = "127.0.0.1:5000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let dsn = "postgres://postgres:postgres@localhost:5432/shortener";
    let state = AppState::try_new(dsn).await?;
    info!("Connected to database: {dsn}");

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {}", LISTEN_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, Error> {
    let id = state.shorten(&data.url).await?;

    let body = Json(ShortenRes {
        url: format!("http://{LISTEN_ADDR}/{id}"),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let url = state.get_url(&id).await?;

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self, Error> {
        let pool = PoolOptions::new();
        let pool = pool.acquire_timeout(Duration::from_secs(5));

        let pool = pool.connect(url).await?;
        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self { db: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String, Error> {
        let id = nanoid!(6);
        let ret = sqlx::query_as::<_, UrlRecord>(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
        )
            .bind(&id)
            .bind(url)
            .fetch_one(&self.db)
            .await;

        if let Err(Database(ref err)) = ret {
            if err.code().is_some_and(|code| code == "23505") {
                // TODO: stackoverflow
                Box::pin(self.shorten(url)).await?;
            }
        }

        Ok(ret?.id)
    }

    async fn get_url(&self, id: &str) -> Result<String, Error> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;

        debug!("get record, id: {id}");
        Ok(ret.url)
    }
}
