use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error")]
    SqlxError(#[from] sqlx::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::SqlxError(sqlx::error::Error::RowNotFound) => {
                StatusCode::NOT_FOUND.into_response()
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        }
    }
}
