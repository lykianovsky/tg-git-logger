use crate::domain::auth::ports::oauth_client::OAuthClientExchangeCodeError;
use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use crate::domain::user::repositories::user_has_roles_repository::AssignRoleToUserError;
use crate::domain::user::repositories::user_repository::CreateUserError;
use crate::domain::user::repositories::user_social_accounts_repository::CreateSocialServiceError;
use crate::domain::user::repositories::user_vc_accounts_repository::CreateVersionControlServiceError;
use crate::domain::version_control::ports::version_control_client::{
    VersionControlClientGetUserError, VersionControlClientOrgMembershipError,
};
use crate::utils::security::crypto::reversible::CipherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterUserViaOAuthExecutorError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User by social user id = {0} already exists")]
    UserBySocialUserIdAlreadyExists(i32),

    #[error("User is not a member of the required organization '{0}'")]
    NotMemberOfRequiredOrganization(String),

    #[error("{0}")]
    OrgMembershipCheckError(#[from] VersionControlClientOrgMembershipError),

    #[error("{0}")]
    OAuthClientExchangeCodeError(#[from] OAuthClientExchangeCodeError),

    #[error("{0}")]
    VersionControlClientError(#[from] VersionControlClientGetUserError),

    #[error("{0}")]
    CreateUserError(#[from] CreateUserError),

    #[error("{0}")]
    CreateSocialServiceError(#[from] CreateSocialServiceError),

    #[error("{0}")]
    CreateVersionControlServiceError(#[from] CreateVersionControlServiceError),

    #[error("{0}")]
    NotificationServiceSendError(#[from] NotificationServiceSendError),

    #[error("{0}")]
    AssignRoleError(#[from] AssignRoleToUserError),

    #[error("{0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    CipherError(#[from] CipherError),
}
