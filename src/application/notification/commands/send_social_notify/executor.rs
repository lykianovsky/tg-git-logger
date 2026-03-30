use crate::application::notification::commands::send_social_notify::command::SendSocialNotifyExecutorCommand;
use crate::application::notification::commands::send_social_notify::error::SendSocialNotifyExecutorError;
use crate::application::notification::commands::send_social_notify::response::SendSocialNotifyExecutorResponse;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct SendSocialNotifyExecutor {
    notification_service: Arc<dyn NotificationService>,
}

impl SendSocialNotifyExecutor {
    pub fn new(notification_service: Arc<dyn NotificationService>) -> Self {
        Self {
            notification_service,
        }
    }
}

impl CommandExecutor for SendSocialNotifyExecutor {
    type Command = SendSocialNotifyExecutorCommand;
    type Response = SendSocialNotifyExecutorResponse;
    type Error = SendSocialNotifyExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.notification_service
            .send_message(&cmd.social_type, &cmd.chat_id, &cmd.message)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    chat_id = %cmd.chat_id.0,
                    social_type = ?cmd.social_type,
                    error = %e,
                    "Failed to send notification"
                );
            })?;

        Ok(SendSocialNotifyExecutorResponse {})
    }
}
