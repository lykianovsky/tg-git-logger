use crate::domain::digest::repositories::digest_subscription_repository::CreateDigestSubscriptionError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateDigestSubscriptionExecutorError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for CreateDigestSubscriptionExecutorError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::UserNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<CreateDigestSubscriptionError> for CreateDigestSubscriptionExecutorError {
    fn from(e: CreateDigestSubscriptionError) -> Self {
        match e {
            CreateDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}
