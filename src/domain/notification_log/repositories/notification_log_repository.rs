use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationLogError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait NotificationLogRepository: Send + Sync {
    async fn was_sent_within(
        &self,
        user_id: UserId,
        kind: &str,
        key: &str,
        since: DateTime<Utc>,
    ) -> Result<bool, NotificationLogError>;

    async fn record_sent(
        &self,
        user_id: UserId,
        kind: &str,
        key: &str,
    ) -> Result<(), NotificationLogError>;
}
