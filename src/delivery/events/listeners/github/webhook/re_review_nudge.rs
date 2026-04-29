use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::pr_review::repositories::pr_review_repository::PrReviewRepository;
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

const NUDGE_KIND: &str = "re_review_nudge";

pub struct WebhookPrReReviewNudgeListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub pr_review_repo: Arc<dyn PrReviewRepository>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub dedup_hours: i64,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPrReReviewNudgeListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        if payload.action != WebhookPullRequestEventActionType::Synchronize {
            return;
        }

        let reviews = match self
            .pr_review_repo
            .find_by_pr(&payload.repo, payload.number)
            .await
        {
            Ok(rs) => rs,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load pr_reviews for re-review nudge");
                return;
            }
        };

        if reviews.is_empty() {
            return;
        }

        let dedup_key = format!("{}:{}", payload.repo, payload.number);
        let since = Utc::now() - Duration::hours(self.dedup_hours);

        for review in reviews {
            if review.reviewer_login.eq_ignore_ascii_case(&payload.author) {
                continue;
            }

            let vc_account = match self
                .user_vc_accounts_repo
                .find_by_login(&review.reviewer_login)
                .await
            {
                Ok(a) => a,
                Err(_) => continue,
            };

            let social_account = match self
                .user_socials_repo
                .find_by_user_id(&vc_account.user_id)
                .await
            {
                Ok(a) => a,
                Err(_) => continue,
            };

            match self
                .notification_log_repo
                .was_sent_within(vc_account.user_id, NUDGE_KIND, &dedup_key, since)
                .await
            {
                Ok(true) => {
                    tracing::debug!(
                        user_id = vc_account.user_id.0,
                        pr = payload.number,
                        "Re-review nudge skipped (cooldown active)"
                    );
                    continue;
                }
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(error = %e, "Failed dedup check, skipping nudge");
                    continue;
                }
            }

            let pr_url = payload.pr_url.as_deref().unwrap_or("");
            let mut msg = MessageBuilder::new()
                .bold(&t!("telegram_bot.notifications.re_review_nudge.title").to_string())
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.re_review_nudge.pr").to_string(),
                    &format!("#{} — {}", payload.number, payload.title),
                )
                .section(
                    &t!("telegram_bot.notifications.re_review_nudge.author").to_string(),
                    &payload.author,
                )
                .section(
                    &t!("telegram_bot.notifications.re_review_nudge.repository").to_string(),
                    &payload.repo,
                )
                .with_html_escape(false);

            if !pr_url.is_empty() {
                msg = msg.empty_line().raw(&format!(
                    "<a href=\"{}\">{}</a>",
                    MessageBuilder::escape_html(pr_url),
                    t!("telegram_bot.notifications.re_review_nudge.open").to_string()
                ));
            }

            tracing::debug!(
                pr = payload.number,
                reviewer = %review.reviewer_login,
                "Sending re-review nudge"
            );

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id: social_account.social_chat_id,
                    message: msg,
                })
                .await
                .ok();

            if let Err(e) = self
                .notification_log_repo
                .record_sent(vc_account.user_id, NUDGE_KIND, &dedup_key)
                .await
            {
                tracing::warn!(error = %e, "Failed to record re-review nudge log");
            }
        }
    }
}
