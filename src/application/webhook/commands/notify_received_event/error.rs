use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyReceivedWebhookEventExecutorError {
    #[error("{0}")]
    NotificationServiceSendError(#[from] NotificationServiceSendError),
}
