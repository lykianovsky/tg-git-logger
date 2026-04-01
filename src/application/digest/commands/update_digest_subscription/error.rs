use crate::domain::digest::repositories::digest_subscription_repository::{
    FindDigestSubscriptionError, UpdateDigestSubscriptionError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateDigestSubscriptionExecutorError {
    #[error("Subscription not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindDigestSubscriptionError> for UpdateDigestSubscriptionExecutorError {
    fn from(e: FindDigestSubscriptionError) -> Self {
        match e {
            FindDigestSubscriptionError::NotFound => Self::NotFound,
            FindDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpdateDigestSubscriptionError> for UpdateDigestSubscriptionExecutorError {
    fn from(e: UpdateDigestSubscriptionError) -> Self {
        match e {
            UpdateDigestSubscriptionError::NotFound => Self::NotFound,
            UpdateDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}
