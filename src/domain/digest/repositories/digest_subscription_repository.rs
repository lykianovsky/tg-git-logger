use crate::domain::digest::entities::digest_subscription::DigestSubscription;
use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;
use crate::domain::user::value_objects::user_id::UserId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateDigestSubscriptionError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindDigestSubscriptionError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Digest subscription not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateDigestSubscriptionError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Digest subscription not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteDigestSubscriptionError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Digest subscription not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait DigestSubscriptionRepository: Send + Sync {
    async fn create(
        &self,
        subscription: &DigestSubscription,
    ) -> Result<DigestSubscription, CreateDigestSubscriptionError>;

    async fn find_by_id(
        &self,
        id: DigestSubscriptionId,
    ) -> Result<DigestSubscription, FindDigestSubscriptionError>;

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DigestSubscription>, FindDigestSubscriptionError>;

    async fn find_due(
        &self,
        hour: i8,
        minute: i8,
    ) -> Result<Vec<DigestSubscription>, FindDigestSubscriptionError>;

    async fn update(
        &self,
        subscription: &DigestSubscription,
    ) -> Result<DigestSubscription, UpdateDigestSubscriptionError>;

    async fn delete(&self, id: DigestSubscriptionId) -> Result<(), DeleteDigestSubscriptionError>;
}
