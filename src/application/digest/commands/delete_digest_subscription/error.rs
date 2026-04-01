use crate::domain::digest::repositories::digest_subscription_repository::DeleteDigestSubscriptionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteDigestSubscriptionExecutorError {
    #[error("Subscription not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<DeleteDigestSubscriptionError> for DeleteDigestSubscriptionExecutorError {
    fn from(e: DeleteDigestSubscriptionError) -> Self {
        match e {
            DeleteDigestSubscriptionError::NotFound => Self::NotFound,
            DeleteDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}
