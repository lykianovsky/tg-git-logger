use crate::application::webhook::commands::notify_received_event::command::NotifyReceivedWebhookEventExecutorCommand;
use crate::application::webhook::commands::notify_received_event::executor::NotifyReceivedWebhookEventExecutor;
use crate::domain::shared::events::consumer::{EventConsumer, EventConsumerError};
use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::shared::events::retry_policy::RetryPolicy;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request::WebhookPullRequestEvent;
use crate::domain::webhook::events::WebhookEvent;
use async_trait::async_trait;
use std::sync::Arc;

pub struct RabbitMQWebhookPullRequestConsumer {
    pub chat_id: SocialChatId,
    pub social_type: SocialType,
    pub notify_received_webhook_event: Arc<NotifyReceivedWebhookEventExecutor>,
}

#[async_trait]
impl EventConsumer for RabbitMQWebhookPullRequestConsumer {
    type EventPayload = WebhookPullRequestEvent;

    fn routing_key(&self) -> &'static str {
        <Self::EventPayload>::EVENT_NAME
    }

    fn queue_name(&self) -> &'static str {
        "webhook.pull_request.handle"
    }

    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy {
            max_attempts: 2,
            delay_ms: 2000,
        }
    }

    async fn handle(&self, payload: WebhookPullRequestEvent) -> Result<(), EventConsumerError> {
        self.notify_received_webhook_event
            .execute(NotifyReceivedWebhookEventExecutorCommand {
                chat_id: self.chat_id,
                social_type: self.social_type,
                message: payload.build_text(),
            })
            .await
            .map_err(|_| {
                EventConsumerError::HandlerFailed("Failed to send notification".to_string())
            })?;
        Ok(())
    }
}
