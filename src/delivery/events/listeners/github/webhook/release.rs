use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::release::WebhookReleaseEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookReleaseEventListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

#[async_trait]
impl EventListener<WebhookReleaseEvent> for WebhookReleaseEventListener {
    async fn handle(&self, payload: &WebhookReleaseEvent) {
        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id: SocialChatId(-5143156647),
                message: MessageBuilder::new().raw(payload.build_text().as_str()),
            })
            .await
            .ok();
    }
}
