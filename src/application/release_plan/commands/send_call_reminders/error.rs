use thiserror::Error;

#[derive(Debug, Error)]
pub enum SendCallRemindersExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
