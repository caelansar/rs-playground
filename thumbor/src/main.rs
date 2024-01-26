use accept_header::Accept;
use anyhow::Result;
use axum::{
    extract::Path,
    http::header::ACCEPT,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    serve, Extension, Router,
};
use bytes::Bytes;
use image::ImageOutputFormat;
use lru::LruCache;
use mime::Mime;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::info;

mod engine;
mod pb;

use pb::*;

use crate::engine::{Engine, Photon};

#[derive(Deserialize)]
struct Params {
    spec: String,
    url: String,
}

type Cache = Arc<Mutex<LruCache<u64, Bytes>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cache: Cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())));

    let app = Router::new()
        .route("/", get(root))
        .route("/image/:spec/:url", get(generate))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(cache)),
        );

    let listener = TcpListener::bind("127.0.0.1:5001").await.unwrap();

    let server = serve(listener, app.into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn root() -> &'static str {
    "Pic"
}

struct OutputFormat(ImageOutputFormat);

impl From<Mime> for OutputFormat {
    fn from(value: Mime) -> Self {
        match value.to_string().as_str() {
            "image/png" => OutputFormat(ImageOutputFormat::Png),
            "image/apng" => OutputFormat(ImageOutputFormat::Png),
            "image/jpeg" => OutputFormat(ImageOutputFormat::Jpeg(85)),
            "image/webp" => OutputFormat(ImageOutputFormat::WebP),
            _ => OutputFormat(ImageOutputFormat::Jpeg(85)),
        }
    }
}

fn get_format(headers: HeaderMap) -> OutputFormat {
    let accept_header = headers.get(ACCEPT).map(|v| v.as_bytes());
    let mut format = OutputFormat(ImageOutputFormat::Png);
    if let Some(accept_header) = accept_header {
        let str_accept = String::from_utf8_lossy(accept_header);
        let accept: Accept = str_accept.parse().unwrap();
        println!("accepts {:?}", accept);

        let available: Vec<Mime> = vec![
            "image/png".parse().unwrap(),
            "image/apng".parse().unwrap(),
            "image/jpeg".parse().unwrap(),
            "image/webp".parse().unwrap(),
        ];

        let negotiated = accept.negotiate(&available).unwrap();
        println!("negotiated: {}", negotiated);

        format = negotiated.into();
    }
    format
}

async fn generate(
    Path(Params { spec, url }): Path<Params>,
    Extension(cache): Extension<Cache>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let url = percent_decode_str(&url).decode_utf8_lossy();
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let img = retrieve_image(&url, cache)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut engine: Photon = img
        .try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    engine.apply(&spec.specs);

    let format = get_format(headers);
    println!("output format: {:?}", format.0);
    let img = engine.generate(format.0);

    info!("done, image size {}", img.len());

    let mut headers = HeaderMap::new();

    headers.insert("content-type", HeaderValue::from_static("image/jpeg"));
    Ok((headers, img.to_vec()))
}

async fn retrieve_image(url: &str, cache: Cache) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);

    let key = hasher.finish();

    let g = &mut cache.lock().await;
    let data = match g.get(&key) {
        Some(v) => {
            info!("get from cache {}", key);
            v.to_owned()
        }
        None => {
            info!("send request...");
            let resp = reqwest::get(url).await?;
            let data = resp.bytes().await?;
            g.put(key, data.clone());
            data
        }
    };
    info!("get data success");
    Ok(data)
}
