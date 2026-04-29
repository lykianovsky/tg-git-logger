use crate::domain::pending_notification::entities::pending_notification::PendingNotification;
use crate::domain::pending_notification::value_objects::pending_notification_id::PendingNotificationId;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreatePendingNotificationError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindPendingNotificationError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum DeletePendingNotificationError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait PendingNotificationsRepository: Send + Sync {
    async fn create(
        &self,
        notification: &PendingNotification,
    ) -> Result<PendingNotification, CreatePendingNotificationError>;

    async fn find_due(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<PendingNotification>, FindPendingNotificationError>;

    async fn delete_many(
        &self,
        ids: &[PendingNotificationId],
    ) -> Result<(), DeletePendingNotificationError>;
}
