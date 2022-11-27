use anyhow::Result;
use axum::{
    extract::Path,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Extension, Router, Server,
};
use bytes::Bytes;
use image::ImageOutputFormat;
use lru::LruCache;
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Arc,
};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};
use tracing::{debug, info};

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

    let addr = "127.0.0.1:5001".parse().unwrap();

    debug!("listening on {}", addr);

    let server = Server::bind(&addr).serve(app.into_make_service());

    if let Err(err) = server.await {
        eprintln!("server error: {}", err);
    }
}

async fn root() -> &'static str {
    "Pic"
}

async fn generate(
    Path(Params { spec, url }): Path<Params>,
    Extension(cache): Extension<Cache>,
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

    let img = engine.generate(ImageOutputFormat::Jpeg(85));

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
