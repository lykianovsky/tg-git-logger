use crate::delivery::events::listeners::github::webhook::resolve_chat_id;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::events::workflow::WebhookWorkflowEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookWorkflowEventListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookWorkflowEvent> for WebhookWorkflowEventListener {
    async fn handle(&self, payload: &WebhookWorkflowEvent) {
        tracing::debug!(
            repo = %payload.repo,
            workflow = %payload.name,
            status = %payload.status,
            conclusion = ?payload.conclusion,
            "Workflow webhook event received"
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
