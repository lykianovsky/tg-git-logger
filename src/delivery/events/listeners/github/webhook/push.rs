use crate::delivery::events::listeners::github::webhook::resolve_chat_id;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::events::push::WebhookPushEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPushEventListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookPushEvent> for WebhookPushEventListener {
    async fn handle(&self, payload: &WebhookPushEvent) {
        tracing::debug!(
            repo = %payload.repo,
            r#ref = %payload.ref_field,
            pusher = %payload.source,
            "Push webhook event received"
        );

        let chat_id =
            resolve_chat_id(&self.repository_repo, &payload.repo, self.default_chat_id).await;

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id,
                message: MessageBuilder::new().raw(payload.build_text().as_str()),
            })
            .await
            .ok();
    }
}
