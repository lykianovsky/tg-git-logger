use crate::domain::pending_notification::repositories::pending_notification_repository::{
    DeletePendingNotificationError, FindPendingNotificationError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlushPendingNotificationsExecutorError {
    #[error("{0}")]
    FindPendingNotificationError(#[from] FindPendingNotificationError),

    #[error("{0}")]
    DeletePendingNotificationError(#[from] DeletePendingNotificationError),
}
