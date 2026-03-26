use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use crate::domain::user::repositories::user_vc_accounts_repository::FindVersionControlServiceByUserIdError;
use crate::domain::version_control::ports::version_control_client::VersionControlClientDateRangeReportError;
use crate::utils::security::crypto::reversible::CipherError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildVersionControlDateRangeReportExecutorError {
    #[error("{0}")]
    VersionControlClientDateRangeReportError(#[from] VersionControlClientDateRangeReportError),

    #[error("{0}")]
    CipherError(#[from] CipherError),

    #[error("{0}")]
    FindVersionControlServiceByUserIdError(#[from] FindVersionControlServiceByUserIdError),

    #[error("{0}")]
    FindSocialServiceByIdError(#[from] FindSocialServiceByIdError),

    #[error("{0}")]
    FindRepositoryByIdError(#[from] FindRepositoryByIdError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("{0}")]
    Serialization(#[from] serde_json::Error),
}
