use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetRepositoryNotificationChatError {
    #[error("Repository not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}
