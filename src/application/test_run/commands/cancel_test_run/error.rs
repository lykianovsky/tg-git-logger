use crate::application::test_run::service::ci_token::ResolveCiTokenError;
use crate::domain::test_run::ports::test_runner::DispatchTestRunError as RunnerDispatchError;
use crate::domain::test_run::repositories::test_run_repository::{
    FindTestRunError, UpdateTestRunError,
};
use crate::domain::test_run::repositories::test_suite_repository::FindTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CancelTestRunError {
    #[error("Database error: {0}")]
    DbError(String),

    /// Отменять нечего: прогон уже завершился или его не было
    #[error("No active test run")]
    NoActiveRun,

    #[error("Tests are not configured for repository")]
    NotConfigured,

    #[error("No linked version control account")]
    NoVersionControlAccount,

    #[error("CI error: {0}")]
    ProviderError(String),
}

impl From<FindTestRunError> for CancelTestRunError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NoActiveRun,
        }
    }
}

impl From<UpdateTestRunError> for CancelTestRunError {
    fn from(error: UpdateTestRunError) -> Self {
        match error {
            UpdateTestRunError::DbError(message) => Self::DbError(message),
        }
    }
}

impl From<FindTestSuiteError> for CancelTestRunError {
    fn from(error: FindTestSuiteError) -> Self {
        match error {
            FindTestSuiteError::DbError(message) => Self::DbError(message),
            FindTestSuiteError::NotConfigured => Self::NotConfigured,
        }
    }
}

impl From<RunnerDispatchError> for CancelTestRunError {
    fn from(error: RunnerDispatchError) -> Self {
        match error {
            RunnerDispatchError::ProviderError(message) => Self::ProviderError(message),
            RunnerDispatchError::WorkflowNotFound(file) => Self::ProviderError(file),
        }
    }
}

impl From<ResolveCiTokenError> for CancelTestRunError {
    fn from(error: ResolveCiTokenError) -> Self {
        match error {
            ResolveCiTokenError::NoVersionControlAccount => Self::NoVersionControlAccount,
            ResolveCiTokenError::DecryptError(message) => Self::DbError(message),
        }
    }
}
