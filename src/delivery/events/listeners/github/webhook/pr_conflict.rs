use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request::{
    WebhookPullRequestEvent, WebhookPullRequestEventActionType,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;

const KIND: &str = "pr_conflict";
const COOLDOWN_HOURS: i64 = 24;

pub struct WebhookPrConflictDetectedListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPrConflictDetectedListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        if !matches!(
            payload.action,
            WebhookPullRequestEventActionType::Opened
                | WebhookPullRequestEventActionType::ReadyForReview
                | WebhookPullRequestEventActionType::Synchronize
                | WebhookPullRequestEventActionType::Reopened
        ) {
            return;
        }

        if payload.mergeable_state.as_deref() != Some("dirty") {
            return;
        }

        let vc_account = match self
            .user_vc_accounts_repo
            .find_by_login(&payload.author)
            .await
        {
            Ok(a) => a,
            Err(_) => return,
        };
        let social = match self
            .user_socials_repo
            .find_by_user_id(&vc_account.user_id)
            .await
        {
            Ok(s) => s,
            Err(_) => return,
        };

        let key = format!("{}:{}", payload.repo, payload.number);
        let since = Utc::now() - Duration::hours(COOLDOWN_HOURS);

        match self
            .notification_log_repo
            .was_sent_within(vc_account.user_id, KIND, &key, since)
            .await
        {
            Ok(true) => {
                tracing::debug!(
                    pr = payload.number,
                    user_id = vc_account.user_id.0,
                    "Conflict notification skipped (cooldown active)"
                );
                return;
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Failed dedup check for conflict notification");
                return;
            }
        }

        let pr_url = payload.pr_url.as_deref().unwrap_or("");
        let mut msg = MessageBuilder::new()
            .bold(&t!("telegram_bot.notifications.pr_conflict.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.pr_conflict.pr").to_string(),
                &format!("#{} — {}", payload.number, payload.title),
            )
            .section(
                &t!("telegram_bot.notifications.pr_conflict.repository").to_string(),
                &payload.repo,
            )
            .with_html_escape(false);

        if !pr_url.is_empty() {
            msg = msg.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url),
                t!("telegram_bot.notifications.pr_conflict.open").to_string()
            ));
        }

        tracing::info!(
            pr = payload.number,
            author = %payload.author,
            "Sending PR conflict DM"
        );

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
            .record_sent(vc_account.user_id, KIND, &key)
            .await
        {
            tracing::warn!(error = %e, "Failed to record conflict notification");
        }
    }
}
