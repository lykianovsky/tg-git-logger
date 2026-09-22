use crate::application::test_run::service::ci_token::ResolveCiTokenError;
use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::test_run::ports::test_runner::ListTestBlocksError as ProviderListError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListCiOptionsError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Repository not found")]
    NotFound,

    #[error("No linked version control account")]
    NoVersionControlAccount,

    #[error("CI error: {0}")]
    ProviderError(String),
}

impl From<FindRepositoryByIdError> for ListCiOptionsError {
    fn from(error: FindRepositoryByIdError) -> Self {
        match error {
            FindRepositoryByIdError::DbError(message) => Self::DbError(message),
            FindRepositoryByIdError::NotFound => Self::NotFound,
        }
    }
}

impl From<ProviderListError> for ListCiOptionsError {
    fn from(error: ProviderListError) -> Self {
        match error {
            ProviderListError::ProviderError(message) => Self::ProviderError(message),
        }
    }
}

impl From<ResolveCiTokenError> for ListCiOptionsError {
    fn from(error: ResolveCiTokenError) -> Self {
        match error {
            ResolveCiTokenError::NoVersionControlAccount => Self::NoVersionControlAccount,
            ResolveCiTokenError::DecryptError(message) => Self::DbError(message),
        }
    }
}
