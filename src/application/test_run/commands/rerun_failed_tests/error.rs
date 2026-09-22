use crate::application::test_run::commands::dispatch_test_run::error::DispatchTestRunError;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RerunFailedTestsError {
    #[error("Database error: {0}")]
    DbError(String),

    /// Перезапускать нечего: прогонов не было или все тесты прошли
    #[error("No failed tests to rerun")]
    NothingToRerun,

    #[error("Test run is already in progress")]
    AlreadyRunning(Box<TestRun>),

    #[error("Tests are not configured for repository")]
    NotConfigured,

    #[error("No linked version control account")]
    NoVersionControlAccount,

    #[error("CI error: {0}")]
    ProviderError(String),
}

impl From<FindTestRunError> for RerunFailedTestsError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NothingToRerun,
        }
    }
}

impl From<DispatchTestRunError> for RerunFailedTestsError {
    fn from(error: DispatchTestRunError) -> Self {
        match error {
            DispatchTestRunError::DbError(message) => Self::DbError(message),
            DispatchTestRunError::AlreadyRunning(run) => Self::AlreadyRunning(run),
            DispatchTestRunError::NotConfigured => Self::NotConfigured,
            DispatchTestRunError::NoVersionControlAccount => Self::NoVersionControlAccount,
            DispatchTestRunError::ProviderError(message) => Self::ProviderError(message),
            DispatchTestRunError::WorkflowNotFound(file) => Self::ProviderError(file),
        }
    }
}
