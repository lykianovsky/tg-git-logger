use crate::domain::digest::repositories::digest_subscription_repository::FindDigestSubscriptionError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUserDigestSubscriptionsError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for GetUserDigestSubscriptionsError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::UserNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<FindDigestSubscriptionError> for GetUserDigestSubscriptionsError {
    fn from(e: FindDigestSubscriptionError) -> Self {
        match e {
            FindDigestSubscriptionError::NotFound => Self::DbError("Not found".to_string()),
            FindDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}
