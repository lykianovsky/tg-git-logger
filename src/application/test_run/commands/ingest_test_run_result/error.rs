use crate::domain::test_run::ports::test_runner::FetchTestRunError;
use crate::domain::test_run::repositories::test_run_repository::{
    FindTestRunError, UpdateTestRunError,
};
use crate::domain::test_run::repositories::test_suite_repository::FindTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestTestRunResultError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Test run not found")]
    NotFound,

    #[error("Tests are not configured for repository")]
    NotConfigured,

    #[error("CI error: {0}")]
    ProviderError(String),
}

impl From<FindTestRunError> for IngestTestRunResultError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NotFound,
        }
    }
}

impl From<UpdateTestRunError> for IngestTestRunResultError {
    fn from(error: UpdateTestRunError) -> Self {
        match error {
            UpdateTestRunError::DbError(message) => Self::DbError(message),
        }
    }
}

impl From<FindTestSuiteError> for IngestTestRunResultError {
    fn from(error: FindTestSuiteError) -> Self {
        match error {
            FindTestSuiteError::DbError(message) => Self::DbError(message),
            FindTestSuiteError::NotConfigured => Self::NotConfigured,
        }
    }
}

impl From<FetchTestRunError> for IngestTestRunResultError {
    fn from(error: FetchTestRunError) -> Self {
        match error {
            FetchTestRunError::ProviderError(message) => Self::ProviderError(message),
            FetchTestRunError::NotFound => Self::NotFound,
        }
    }
}
