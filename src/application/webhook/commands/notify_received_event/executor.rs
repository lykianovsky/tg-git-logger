use crate::application::webhook::commands::notify_received_event::command::NotifyReceivedWebhookEventExecutorCommand;
use crate::application::webhook::commands::notify_received_event::error::NotifyReceivedWebhookEventExecutorError;
use crate::application::webhook::commands::notify_received_event::response::NotifyReceivedWebhookEventExecutorResponse;
use crate::domain::notification::services::notification_service::NotificationService;
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

    pub async fn execute(
        &self,
        cmd: NotifyReceivedWebhookEventExecutorCommand,
    ) -> Result<NotifyReceivedWebhookEventExecutorResponse, NotifyReceivedWebhookEventExecutorError>
    {
        self.notification_service
            .send(&cmd.social_type, &cmd.chat_id, &cmd.message)
            .await?;

        Ok(NotifyReceivedWebhookEventExecutorResponse {})
    }
}
