use crate::application::webhook::commands::notify_received_event::command::NotifyReceivedWebhookEventExecutorCommand;
use crate::application::webhook::commands::notify_received_event::error::NotifyReceivedWebhookEventExecutorError;
use crate::application::webhook::commands::notify_received_event::response::NotifyReceivedWebhookEventExecutorResponse;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct NotifyReceivedWebhookEventExecutor {
    notification_service: Arc<dyn NotificationService>,
}

impl NotifyReceivedWebhookEventExecutor {
    pub fn new(notification_service: Arc<dyn NotificationService>) -> Self {
        Self {
            notification_service,
        }
    }
}

impl CommandExecutor for NotifyReceivedWebhookEventExecutor {
    type Command = NotifyReceivedWebhookEventExecutorCommand;
    type Response = NotifyReceivedWebhookEventExecutorResponse;
    type Error = NotifyReceivedWebhookEventExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.notification_service
            .send(&cmd.social_type, &cmd.chat_id, &cmd.message)
            .await?;

        Ok(NotifyReceivedWebhookEventExecutorResponse {})
    }
}
