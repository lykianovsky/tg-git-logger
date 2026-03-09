use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdException;
use crate::domain::user::repositories::user_vc_accounts_repository::FindVersionControlServiceByUserIdException;
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
    FindVersionControlServiceByUserIdException(#[from] FindVersionControlServiceByUserIdException),

    #[error("{0}")]
    FindSocialServiceByIdException(#[from] FindSocialServiceByIdException),
}
