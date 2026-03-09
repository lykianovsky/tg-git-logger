use crate::application::webhook::commands::dispatch_event::command::DispatchWebhookEventExecutorCommand;
use crate::application::webhook::commands::dispatch_event::error::DispatchWebhookEventExecutorError;
use crate::application::webhook::commands::dispatch_event::response::DispatchWebhookEventExecutorResponse;
use crate::bootstrap::TestEvent;
use crate::delivery::jobs::consumers::send_email::payload::SendEmailJob;
use crate::domain::shared::command::CommandExecutor;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerPublisher,
};
use std::sync::Arc;

pub struct DispatchWebhookEventExecutor {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

impl DispatchWebhookEventExecutor {}

impl CommandExecutor for DispatchWebhookEventExecutor {
    type Command = DispatchWebhookEventExecutorCommand;
    type Response = DispatchWebhookEventExecutorResponse;
    type Error = DispatchWebhookEventExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        tracing::debug!("Dispatching webhook event");

        self.publisher
            .publish(&TestEvent {
                keys: "keys??????".to_string(),
            })
            .await
            .ok();

        self.publisher
            .publish(&SendEmailJob {
                email: "test.email".to_string(),
            })
            .await
            .ok();

        Ok(DispatchWebhookEventExecutorResponse {})
    }
}
