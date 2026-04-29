use crate::delivery::events::listeners::github::webhook::resolve_chat_id;
use crate::delivery::jobs::consumers::move_task_to_test::payload::MoveTaskToTestJob;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::repository::repositories::repository_task_tracker_repository::RepositoryTaskTrackerRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::task::value_objects::task_id::TaskId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::events::pull_request::{
    WebhookPullRequestEvent, WebhookPullRequestEventActionType,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPullRequestEventListener {
    pub task_tracker_service: Arc<dyn TaskTrackerService>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    pub default_chat_id: SocialChatId,
}

impl WebhookPullRequestEventListener {
    async fn extract_task_id_and_column(&self, repo: &str, title: &str) -> Option<(TaskId, u64)> {
        let mut parts = repo.splitn(2, '/');
        let (owner, name) = match (parts.next(), parts.next()) {
            (Some(o), Some(n)) => (o, n),
            _ => return None,
        };

        let repository = self
            .repository_repo
            .find_by_owner_and_name(owner, name)
            .await
            .ok()?;

        let tracker = self
            .repository_task_tracker_repo
            .find_by_repository_id(repository.id)
            .await
            .ok()?;

        let column_id = tracker.qa_column_id as u64;

        self.task_tracker_service
            .extract_match_with_pattern(title, &tracker.extract_pattern_regexp)
            .map(|(_, task_id)| (task_id, column_id))
    }
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPullRequestEventListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        tracing::debug!(
            repo = %payload.repo,
            pr = payload.number,
            action = ?payload.action,
            merged = payload.merged,
            "PR webhook event received"
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

        if payload.merged && payload.action == WebhookPullRequestEventActionType::Closed {
            if let Some((task_id, column_id)) = self
                .extract_task_id_and_column(&payload.repo, &payload.title)
                .await
            {
                tracing::debug!(
                    task_id = %task_id.0,
                    column_id = %column_id,
                    pr = payload.number,
                    "Task extracted from merged PR, scheduling move to test"
                );
                self.publisher
                    .publish(&MoveTaskToTestJob { task_id, column_id })
                    .await
                    .ok();
            }
        }
    }
}
