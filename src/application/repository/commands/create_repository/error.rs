use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateRepositoryExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
