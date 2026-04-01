use thiserror::Error;

#[derive(Debug, Error)]
pub enum SendDueDigestsExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
