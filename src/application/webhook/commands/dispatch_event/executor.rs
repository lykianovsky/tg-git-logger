use crate::application::webhook::commands::dispatch_event::command::DispatchWebhookEventExecutorCommand;
use crate::application::webhook::commands::dispatch_event::error::DispatchWebhookEventExecutorError;
use crate::application::webhook::commands::dispatch_event::response::DispatchWebhookEventExecutorResponse;
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
            .publish(cmd.event.as_ref() as &dyn MessageBrokerMessage)
            .await
            .ok();

        Ok(DispatchWebhookEventExecutorResponse {})
    }
}
