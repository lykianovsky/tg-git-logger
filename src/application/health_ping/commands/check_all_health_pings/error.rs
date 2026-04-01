use crate::domain::health_ping::repositories::health_ping_repository::{
    FindHealthPingError, UpdateHealthPingError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckAllHealthPingsExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindHealthPingError> for CheckAllHealthPingsExecutorError {
    fn from(e: FindHealthPingError) -> Self {
        Self::DbError(e.to_string())
    }
}

impl From<UpdateHealthPingError> for CheckAllHealthPingsExecutorError {
    fn from(e: UpdateHealthPingError) -> Self {
        Self::DbError(e.to_string())
    }
}
