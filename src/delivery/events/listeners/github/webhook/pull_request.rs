use crate::delivery::jobs::consumers::move_task_to_test::payload::MoveTaskToTestJob;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request::WebhookPullRequestEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPullRequestEventListener {
    pub task_tracker_service: Arc<dyn TaskTrackerService>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPullRequestEventListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id: self.chat_id,
                message: MessageBuilder::new().raw(payload.build_text().as_str()),
            })
            .await
            .ok();

        if payload.merged {
            if let Some(task_id) = self
                .task_tracker_service
                .extract_task_id_by_pattern(&payload.title)
            {
                self.publisher
                    .publish(&MoveTaskToTestJob { task_id })
                    .await
                    .ok();
            }
        }
    }
}
