use crate::domain::pending_notification::repositories::pending_notification_repository::CreatePendingNotificationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferNotificationExecutorError {
    #[error("{0}")]
    CreatePendingNotificationError(#[from] CreatePendingNotificationError),
}
