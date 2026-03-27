use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnsetRepositoryNotificationChatError {
    #[error("Repository not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}
