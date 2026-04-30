use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::workflow::WebhookWorkflowEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;

const KIND: &str = "ci_fail";
const COOLDOWN_HOURS: i64 = 24;

pub struct WebhookCiFailDmListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

#[async_trait]
impl EventListener<WebhookWorkflowEvent> for WebhookCiFailDmListener {
    async fn handle(&self, payload: &WebhookWorkflowEvent) {
        if payload.status != "completed" {
            return;
        }
        if payload.conclusion.as_deref() != Some("failure") {
            return;
        }

        let actor_login = match &payload.actor {
            Some(login) if !login.is_empty() => login.clone(),
            _ => return,
        };

        let vc_account = match self.user_vc_accounts_repo.find_by_login(&actor_login).await {
            Ok(a) => a,
            Err(_) => {
                tracing::debug!(actor = %actor_login, "CI fail actor not registered — skipping DM");
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

        let key = format!("{}:{}", payload.repo, payload.id);
        let since = Utc::now() - Duration::hours(COOLDOWN_HOURS);

        match self
            .notification_log_repo
            .was_sent_within(vc_account.user_id, KIND, &key, since)
            .await
        {
            Ok(true) => return,
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Failed dedup check for ci_fail notification");
                return;
            }
        }

        let workflow_url = payload.html_url.as_deref().unwrap_or("");
        let mut msg = MessageBuilder::new()
            .bold(&t!("telegram_bot.notifications.ci_fail.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.ci_fail.workflow").to_string(),
                &payload.name,
            )
            .section(
                &t!("telegram_bot.notifications.ci_fail.repository").to_string(),
                &payload.repo,
            )
            .with_html_escape(false);

        if !workflow_url.is_empty() {
            msg = msg.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(workflow_url),
                t!("telegram_bot.notifications.ci_fail.open").to_string()
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
            .record_sent(vc_account.user_id, KIND, &key)
            .await
        {
            tracing::warn!(error = %e, "Failed to record ci_fail notification");
        }
    }
}
