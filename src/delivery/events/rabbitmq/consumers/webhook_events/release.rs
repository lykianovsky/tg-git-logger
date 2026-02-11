use crate::application::webhook::commands::notify_received_event::command::NotifyReceivedWebhookEventExecutorCommand;
use crate::application::webhook::commands::notify_received_event::executor::NotifyReceivedWebhookEventExecutor;
use crate::domain::shared::events::consumer::{EventConsumer, EventConsumerError};
use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::release::WebhookReleaseEvent;
use crate::domain::webhook::events::WebhookEvent;
use async_trait::async_trait;
use std::sync::Arc;

pub struct RabbitMQWebhookReleaseConsumer {
    pub chat_id: SocialChatId,
    pub social_type: SocialType,
    pub notify_received_webhook_event: Arc<NotifyReceivedWebhookEventExecutor>,
}

#[async_trait]
impl EventConsumer for RabbitMQWebhookReleaseConsumer {
    type EventPayload = WebhookReleaseEvent;

    fn routing_key(&self) -> &'static str {
        <Self::EventPayload as StaticDomainEvent>::EVENT_NAME
    }

    fn queue_name(&self) -> &'static str {
        "webhook.release.handle"
    }

    async fn handle(&self, payload: WebhookReleaseEvent) -> Result<(), EventConsumerError> {
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
