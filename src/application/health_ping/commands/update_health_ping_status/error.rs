use crate::domain::health_ping::repositories::health_ping_repository::{
    FindHealthPingError, UpdateHealthPingError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateHealthPingStatusExecutorError {
    #[error("Health ping not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindHealthPingError> for UpdateHealthPingStatusExecutorError {
    fn from(e: FindHealthPingError) -> Self {
        match e {
            FindHealthPingError::NotFound => Self::NotFound,
            FindHealthPingError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpdateHealthPingError> for UpdateHealthPingStatusExecutorError {
    fn from(e: UpdateHealthPingError) -> Self {
        match e {
            UpdateHealthPingError::NotFound => Self::NotFound,
            UpdateHealthPingError::DbError(msg) => Self::DbError(msg),
        }
    }
}
