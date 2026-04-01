use crate::domain::health_ping::repositories::health_ping_repository::DeleteHealthPingError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteHealthPingExecutorError {
    #[error("Health ping not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<DeleteHealthPingError> for DeleteHealthPingExecutorError {
    fn from(e: DeleteHealthPingError) -> Self {
        match e {
            DeleteHealthPingError::NotFound => Self::NotFound,
            DeleteHealthPingError::DbError(msg) => Self::DbError(msg),
        }
    }
}
