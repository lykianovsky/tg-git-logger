use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetLastTestRunError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindTestRunError> for GetLastTestRunError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::DbError("Test run not found".to_string()),
        }
    }
}
