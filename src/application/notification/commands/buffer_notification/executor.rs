use crate::application::notification::commands::buffer_notification::command::BufferNotificationExecutorCommand;
use crate::application::notification::commands::buffer_notification::error::BufferNotificationExecutorError;
use crate::application::notification::commands::buffer_notification::response::BufferNotificationExecutorResponse;
use crate::domain::pending_notification::entities::pending_notification::PendingNotification;
use crate::domain::pending_notification::repositories::pending_notification_repository::PendingNotificationsRepository;
use crate::domain::pending_notification::value_objects::pending_notification_id::PendingNotificationId;
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use std::sync::Arc;

pub struct BufferNotificationExecutor {
    repo: Arc<dyn PendingNotificationsRepository>,
}

impl BufferNotificationExecutor {
    pub fn new(repo: Arc<dyn PendingNotificationsRepository>) -> Self {
        Self { repo }
    }
}

impl CommandExecutor for BufferNotificationExecutor {
    type Command = BufferNotificationExecutorCommand;
    type Response = BufferNotificationExecutorResponse;
    type Error = BufferNotificationExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let pending = PendingNotification {
            id: PendingNotificationId(0),
            user_id: cmd.user_id,
            social_type: cmd.social_type,
            social_chat_id: cmd.chat_id,
            message: cmd.message.clone(),
            event_type: cmd.event_type.clone(),
            deliver_after: cmd.deliver_after,
            created_at: Utc::now(),
        };
        let _ = pending.created_at;

        self.repo.create(&pending).await?;

        Ok(BufferNotificationExecutorResponse {})
    }
}
