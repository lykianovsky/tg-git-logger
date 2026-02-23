use crate::domain::user::repositories::user_social_services_repository::FindSocialServiceByIdException;
use crate::domain::user::repositories::user_version_control_services::FindVersionControlServiceByUserIdException;
use crate::infrastructure::integrations::version_control::github::client::GithubClientError;
use crate::utils::security::crypto::reversible::CipherError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildVersionControlDateRangeReportExecutorError {
    #[error("{0}")]
    GithubClientError(#[from] GithubClientError),

    #[error("{0}")]
    CipherError(#[from] CipherError),

    #[error("{0}")]
    FindVersionControlServiceByUserIdException(#[from] FindVersionControlServiceByUserIdException),

    #[error("{0}")]
    FindSocialServiceByIdException(#[from] FindSocialServiceByIdException),
}
