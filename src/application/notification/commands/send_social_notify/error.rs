use crate::application::notification::commands::buffer_notification::error::BufferNotificationExecutorError;
use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SendSocialNotifyExecutorError {
    #[error("{0}")]
    NotificationServiceSendError(#[from] NotificationServiceSendError),

    #[error("{0}")]
    BufferNotificationError(#[from] BufferNotificationExecutorError),
}
