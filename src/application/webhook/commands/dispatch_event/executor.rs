use crate::application::webhook::commands::dispatch_event::command::DispatchWebhookEventExecutorCommand;
use crate::application::webhook::commands::dispatch_event::error::DispatchWebhookEventExecutorError;
use crate::application::webhook::commands::dispatch_event::response::DispatchWebhookEventExecutorResponse;
use crate::domain::shared::events::publisher::EventPublisher;
use std::sync::Arc;

pub struct DispatchWebhookEventExecutor {
    pub publisher: Arc<dyn EventPublisher>,
}

impl DispatchWebhookEventExecutor {
    pub async fn execute(
        &self,
        cmd: DispatchWebhookEventExecutorCommand,
    ) -> Result<DispatchWebhookEventExecutorResponse, DispatchWebhookEventExecutorError> {
        tracing::debug!("Dispatching webhook event");

        self.publisher.publish(&*cmd.event).await.map_err(|_| {
            tracing::error!("Failed to publish event: {}", cmd.event.event_name());
            DispatchWebhookEventExecutorError::UnknownError
        })?;

        Ok(DispatchWebhookEventExecutorResponse {})
    }
}
