use crate::domain::notification::services::notification_service::NotificationServiceSendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SendSocialNotifyExecutorError {
    #[error("{0}")]
    NotificationServiceSendError(#[from] NotificationServiceSendError),
}
