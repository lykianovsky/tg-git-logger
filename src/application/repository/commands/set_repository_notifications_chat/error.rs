use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetRepositoryNotificationsChatError {
    #[error("Repository not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}
