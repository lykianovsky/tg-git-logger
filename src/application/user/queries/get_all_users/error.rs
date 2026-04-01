use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetAllUsersExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
