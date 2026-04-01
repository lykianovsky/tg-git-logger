use crate::domain::health_ping::repositories::health_ping_repository::FindHealthPingError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetAllHealthPingsError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindHealthPingError> for GetAllHealthPingsError {
    fn from(e: FindHealthPingError) -> Self {
        match e {
            FindHealthPingError::DbError(msg) => Self::DbError(msg),
            FindHealthPingError::NotFound => Self::DbError("Not found".to_string()),
        }
    }
}
