use crate::domain::auth::ports::oauth_client::OAuthClientExchangeCodeError;
use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use crate::domain::user::repositories::user_has_roles_repository::AssignRoleToUserError;
use crate::domain::user::repositories::user_repository::CreateUserError;
use crate::domain::user::repositories::user_social_accounts_repository::CreateSocialServiceError;
use crate::domain::user::repositories::user_vc_accounts_repository::CreateVersionControlServiceError;
use crate::domain::version_control::ports::version_control_client::VersionControlClientGetUserError;
use crate::utils::security::crypto::reversible::CipherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterUserViaOAuthExecutorError {
    #[error("Database error: {0}")]
    DbError(#[from] sea_orm::error::DbErr),

    #[error("User by social user id = {0} already exists")]
    UserBySocialUserIdAlreadyExists(i32),

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

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("{0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    CipherError(#[from] CipherError),

    #[error("Invalid state error")]
    InvalidState,
}
