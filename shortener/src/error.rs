#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error")]
    SqlxError(#[from] sqlx::error::Error),
}
