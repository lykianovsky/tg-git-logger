use crate::domain::health_ping::repositories::health_ping_repository::CreateHealthPingError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateHealthPingExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<CreateHealthPingError> for CreateHealthPingExecutorError {
    fn from(e: CreateHealthPingError) -> Self {
        match e {
            CreateHealthPingError::DbError(msg) => Self::DbError(msg),
        }
    }
}
