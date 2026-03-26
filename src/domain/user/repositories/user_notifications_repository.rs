use crate::domain::user::entities::user_notification::UserNotification;
use crate::domain::user::value_objects::user_id::UserId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateUserNotificationError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindUserNotificationByIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Notification not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindUserNotificationsByUserIdError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait]
pub trait UserNotificationsRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        notification: &UserNotification,
    ) -> Result<UserNotification, CreateUserNotificationError>;

    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<UserNotification, FindUserNotificationByIdError>;

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<UserNotification>, FindUserNotificationsByUserIdError>;
}
