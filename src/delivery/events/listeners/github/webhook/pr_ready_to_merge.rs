use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request_review::{
    WebhookPullRequestReviewEvent, WebhookPullRequestReviewState,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;

const KIND: &str = "pr_ready_to_merge";
const COOLDOWN_HOURS: i64 = 24 * 7;

pub struct WebhookPrReadyToMergeListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

#[async_trait]
impl EventListener<WebhookPullRequestReviewEvent> for WebhookPrReadyToMergeListener {
    async fn handle(&self, payload: &WebhookPullRequestReviewEvent) {
        if !matches!(payload.state, WebhookPullRequestReviewState::Approved) {
            return;
        }

        if payload.mergeable_state.as_deref() != Some("clean") {
            return;
        }

        let admin_user_ids = match self
            .user_has_roles_repo
            .find_user_ids_by_role(RoleName::Admin)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load admin user_ids for ready-to-merge notify");
                return;
            }
        };

        if admin_user_ids.is_empty() {
            return;
        }

        let key = format!("{}:{}", payload.repo, payload.pr_number);
        let since = Utc::now() - Duration::hours(COOLDOWN_HOURS);

        for user_id in &admin_user_ids {
            match self
                .notification_log_repo
                .was_sent_within(*user_id, KIND, &key, since)
                .await
            {
                Ok(true) => continue,
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "Failed dedup check for ready-to-merge notification");
                    continue;
                }
            }

            let social = match self.user_socials_repo.find_by_user_id(user_id).await {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut msg = MessageBuilder::new()
                .bold(&t!("telegram_bot.notifications.pr_ready_to_merge.title").to_string())
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.pr_ready_to_merge.pr").to_string(),
                    &format!("#{} — {}", payload.pr_number, payload.pr_title),
                )
                .section(
                    &t!("telegram_bot.notifications.pr_ready_to_merge.repository").to_string(),
                    &payload.repo,
                )
                .section(
                    &t!("telegram_bot.notifications.pr_ready_to_merge.author").to_string(),
                    &payload.pr_author,
                )
                .with_html_escape(false);

            if !payload.pr_url.is_empty() {
                msg = msg.empty_line().raw(&format!(
                    "<a href=\"{}\">{}</a>",
                    MessageBuilder::escape_html(&payload.pr_url),
                    t!("telegram_bot.notifications.pr_ready_to_merge.open").to_string()
                ));
            }

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id: social.social_chat_id,
                    message: msg,
                })
                .await
                .ok();

            if let Err(e) = self
                .notification_log_repo
                .record_sent(*user_id, KIND, &key)
                .await
            {
                tracing::warn!(error = %e, "Failed to record ready-to-merge notification");
            }
        }
    }
}
