use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUserBoundRepositoriesError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DbError(String),
}
