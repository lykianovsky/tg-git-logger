use crate::domain::auth::ports::oauth_client::OAuthClientExchangeCodeError;
use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use crate::domain::user::repositories::user_repository::CreateUserException;
use crate::domain::user::repositories::user_social_accounts_repository::CreateSocialServiceException;
use crate::domain::user::repositories::user_vc_accounts_repository::CreateVersionControlServiceException;
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
    CreateUserException(#[from] CreateUserException),

    #[error("{0}")]
    CreateSocialServiceException(#[from] CreateSocialServiceException),

    #[error("{0}")]
    CreateVersionControlServiceException(#[from] CreateVersionControlServiceException),

    #[error("{0}")]
    NotificationServiceSendError(#[from] NotificationServiceSendError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("{0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    CipherError(#[from] CipherError),

    #[error("Invalid state error")]
    InvalidState,
}
