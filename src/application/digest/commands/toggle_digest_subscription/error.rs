use crate::domain::digest::repositories::digest_subscription_repository::{
    FindDigestSubscriptionError, UpdateDigestSubscriptionError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToggleDigestSubscriptionExecutorError {
    #[error("Subscription not found")]
    NotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindDigestSubscriptionError> for ToggleDigestSubscriptionExecutorError {
    fn from(e: FindDigestSubscriptionError) -> Self {
        match e {
            FindDigestSubscriptionError::NotFound => Self::NotFound,
            FindDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpdateDigestSubscriptionError> for ToggleDigestSubscriptionExecutorError {
    fn from(e: UpdateDigestSubscriptionError) -> Self {
        match e {
            UpdateDigestSubscriptionError::NotFound => Self::NotFound,
            UpdateDigestSubscriptionError::DbError(msg) => Self::DbError(msg),
        }
    }
}
