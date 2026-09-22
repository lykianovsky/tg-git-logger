use crate::application::test_run::service::ci_token::ResolveCiTokenError;
use crate::domain::test_run::ports::test_runner::FetchTestRunError;
use crate::domain::test_run::repositories::test_suite_repository::FindTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetTestRunProgressError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Tests are not configured for repository")]
    NotConfigured,

    #[error("No linked version control account")]
    NoVersionControlAccount,

    #[error("CI error: {0}")]
    ProviderError(String),
}

impl From<FindTestSuiteError> for GetTestRunProgressError {
    fn from(error: FindTestSuiteError) -> Self {
        match error {
            FindTestSuiteError::DbError(message) => Self::DbError(message),
            FindTestSuiteError::NotConfigured => Self::NotConfigured,
        }
    }
}

impl From<FetchTestRunError> for GetTestRunProgressError {
    fn from(error: FetchTestRunError) -> Self {
        match error {
            FetchTestRunError::ProviderError(message) => Self::ProviderError(message),
            FetchTestRunError::NotFound => Self::ProviderError("Run not found".to_string()),
        }
    }
}

impl From<ResolveCiTokenError> for GetTestRunProgressError {
    fn from(error: ResolveCiTokenError) -> Self {
        match error {
            ResolveCiTokenError::NoVersionControlAccount => Self::NoVersionControlAccount,
            ResolveCiTokenError::DecryptError(message) => Self::DbError(message),
        }
    }
}
