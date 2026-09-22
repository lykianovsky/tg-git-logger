use crate::domain::test_run::repositories::test_failure_card_repository::FindTestFailureCardError;
use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetRunFailuresError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Test run not found")]
    NotFound,
}

impl From<FindTestRunError> for GetRunFailuresError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NotFound,
        }
    }
}

impl From<FindTestFailureCardError> for GetRunFailuresError {
    fn from(error: FindTestFailureCardError) -> Self {
        match error {
            FindTestFailureCardError::DbError(message) => Self::DbError(message),
        }
    }
}
