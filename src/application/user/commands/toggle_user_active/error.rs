use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToggleUserActiveExecutorError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}
