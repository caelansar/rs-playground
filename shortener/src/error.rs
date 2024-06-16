use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error")]
    SqlxError(#[from] sqlx::error::Error),
    #[error("url not found")]
    UrlNotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::UrlNotFound => StatusCode::NOT_FOUND.into_response(),
            Error::SqlxError(_) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        }
    }
}
