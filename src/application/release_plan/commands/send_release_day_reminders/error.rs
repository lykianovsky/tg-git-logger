use thiserror::Error;

#[derive(Debug, Error)]
pub enum SendReleaseDayRemindersExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
