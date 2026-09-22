use crate::delivery::jobs::consumers::move_task_to_test::payload::MoveTaskToTestJob;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::repository::repositories::repository_task_tracker_repository::RepositoryTaskTrackerRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::task::value_objects::task_id::TaskId;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request::{
    WebhookPullRequestEvent, WebhookPullRequestEventActionType,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::{MessageBuilder, build_repo_header};
use async_trait::async_trait;
use std::sync::Arc;

/// При открытии (или выводе из draft / reopen) PR — двигает связанную задачу
/// в review-колонку Kaiten. Если ID задачи не извлекается из title — DM автору.
pub struct WebhookPrMoveToReviewListener {
    pub task_tracker_service: Arc<dyn TaskTrackerService>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

struct TrackerHit {
    task_id: Option<TaskId>,
    review_column_id: u64,
    extract_pattern: String,
}

impl WebhookPrMoveToReviewListener {
    async fn lookup_tracker(&self, repo: &str, title: &str) -> Option<TrackerHit> {
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

        // Если review-колонка не настроена отдельно (равна QA-колонке после backfill миграции),
        // считаем что review-флоу для этой репы не включён, и админ должен явно его настроить.
        if tracker.review_column_id == tracker.qa_column_id {
            return None;
        }

        let task_id = self
            .task_tracker_service
            .extract_match_with_pattern(title, &tracker.extract_pattern_regexp)
            .map(|(_, task_id)| task_id);

        Some(TrackerHit {
            task_id,
            review_column_id: tracker.review_column_id as u64,
            extract_pattern: tracker.extract_pattern_regexp,
        })
    }

    async fn notify_author_missing_task_id(
        &self,
        payload: &WebhookPullRequestEvent,
        extract_pattern: &str,
    ) {
        let vc_account = match self
            .user_vc_accounts_repo
            .find_by_login(&payload.author)
            .await
        {
            Ok(a) => a,
            Err(_) => {
                tracing::debug!(
                    author = %payload.author,
                    pr = payload.number,
                    "PR author not registered in bot — cannot DM about missing task id"
                );
                return;
            }
        };

        let social = match self
            .user_socials_repo
            .find_by_user_id(&vc_account.user_id)
            .await
        {
            Ok(s) => s,
            Err(_) => return,
        };

        let pr_url = payload.pr_url.as_deref().unwrap_or("");
        let repo_line = build_repo_header(&payload.repo, payload.repo_url.as_deref());

        let mut message = MessageBuilder::new()
            .bold(&repo_line)
            .bold(&t!("telegram_bot.notifications.missing_task_id.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.missing_task_id.pr").to_string(),
                &format!("#{} — {}", payload.number, payload.title),
            )
            .section(
                &t!("telegram_bot.notifications.missing_task_id.pattern").to_string(),
                extract_pattern,
            )
            .with_html_escape(false);

        if !pr_url.is_empty() {
            message = message.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url),
                t!("telegram_bot.notifications.missing_task_id.open").to_string()
            ));
        }

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id: social.social_chat_id,
                message,
            })
            .await
            .ok();
    }
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPrMoveToReviewListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        let is_trigger = matches!(
            payload.action,
            WebhookPullRequestEventActionType::Opened
                | WebhookPullRequestEventActionType::ReadyForReview
                | WebhookPullRequestEventActionType::Reopened
                | WebhookPullRequestEventActionType::Edited
        );
        if !is_trigger {
            return;
        }
        if payload.draft {
            tracing::debug!(pr = payload.number, "Skipping move-to-review: PR is draft");
            return;
        }

        let hit = match self.lookup_tracker(&payload.repo, &payload.title).await {
            Some(h) => h,
            None => {
                tracing::debug!(
                    pr = payload.number,
                    repo = %payload.repo,
                    "No task tracker configured for repo — skipping move-to-review"
                );
                return;
            }
        };

        match hit.task_id {
            Some(task_id) => {
                tracing::debug!(
                    task_id = %task_id.0,
                    column_id = hit.review_column_id,
                    pr = payload.number,
                    "Task extracted from opened PR, scheduling move to review"
                );
                self.publisher
                    .publish(&MoveTaskToTestJob {
                        task_id,
                        column_id: hit.review_column_id,
                    })
                    .await
                    .ok();
            }
            None => {
                tracing::info!(
                    pr = payload.number,
                    author = %payload.author,
                    "Task id not found in PR title — DM author"
                );
                self.notify_author_missing_task_id(payload, &hit.extract_pattern)
                    .await;
            }
        }
    }
}
