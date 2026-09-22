use crate::application::test_run::service::ci_token::ResolveCiTokenError;
use crate::domain::test_run::ports::test_runner::ListTestBlocksError as ProviderListError;
use crate::domain::test_run::repositories::test_suite_repository::FindTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListTestBlocksError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Tests are not configured for repository")]
    NotConfigured,

    #[error("CI error: {0}")]
    ProviderError(String),

    #[error("No linked version control account")]
    NoVersionControlAccount,
}

impl From<ResolveCiTokenError> for ListTestBlocksError {
    fn from(error: ResolveCiTokenError) -> Self {
        match error {
            ResolveCiTokenError::NoVersionControlAccount => Self::NoVersionControlAccount,
            ResolveCiTokenError::DecryptError(message) => Self::DbError(message),
        }
    }
}

impl From<FindTestSuiteError> for ListTestBlocksError {
    fn from(error: FindTestSuiteError) -> Self {
        match error {
            FindTestSuiteError::DbError(message) => Self::DbError(message),
            FindTestSuiteError::NotConfigured => Self::NotConfigured,
        }
    }
}

impl From<ProviderListError> for ListTestBlocksError {
    fn from(error: ProviderListError) -> Self {
        match error {
            ProviderListError::ProviderError(message) => Self::ProviderError(message),
        }
    }
}
