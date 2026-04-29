use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::domain::webhook::events::pull_request::{
    WebhookPullRequestEvent, WebhookPullRequestEventActionType,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

pub struct WebhookReviewRequestedDmListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub reversible_cipher: Arc<ReversibleCipher>,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookReviewRequestedDmListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        if payload.action != WebhookPullRequestEventActionType::ReviewRequested {
            return;
        }

        let reviewer_login = match payload.requested_reviewer.as_deref() {
            Some(login) => login,
            None => {
                tracing::debug!(
                    pr = payload.number,
                    "review_requested without requested_reviewer (probably team) — skipping DM"
                );
                return;
            }
        };

        if reviewer_login.eq_ignore_ascii_case(&payload.author) {
            return;
        }

        let vc_account = match self
            .user_vc_accounts_repo
            .find_by_login(reviewer_login)
            .await
        {
            Ok(account) => account,
            Err(_) => {
                tracing::debug!(
                    reviewer = %reviewer_login,
                    "Reviewer not found in DB — skipping DM"
                );
                return;
            }
        };

        let social_account = match self
            .user_socials_repo
            .find_by_user_id(&vc_account.user_id)
            .await
        {
            Ok(account) => account,
            Err(_) => {
                tracing::debug!(
                    user_id = ?vc_account.user_id,
                    "Reviewer has no social account — skipping DM"
                );
                return;
            }
        };

        let prefs = self
            .user_preferences_repo
            .find_by_user_id(vc_account.user_id)
            .await
            .unwrap_or(None);

        let now = Utc::now();
        let vacation_until = prefs.as_ref().and_then(|p| p.vacation_until);

        if let Some(until) = vacation_until {
            if until > now {
                tracing::info!(
                    pr = payload.number,
                    reviewer = %reviewer_login,
                    until = %until,
                    "Reviewer is on vacation — posting comment in PR + alerting admins"
                );
                self.handle_reviewer_on_vacation(payload, reviewer_login, until)
                    .await;
                return;
            }
        }

        let pr_url = payload.pr_url.as_deref().unwrap_or("");
        let mut message = MessageBuilder::new()
            .bold(&t!("telegram_bot.notifications.review_requested.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.review_requested.pr").to_string(),
                &format!("#{} — {}", payload.number, payload.title),
            )
            .section(
                &t!("telegram_bot.notifications.review_requested.author").to_string(),
                &payload.author,
            )
            .section(
                &t!("telegram_bot.notifications.review_requested.repository").to_string(),
                &payload.repo,
            )
            .with_html_escape(false);

        if !pr_url.is_empty() {
            message = message.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url),
                t!("telegram_bot.notifications.review_requested.open").to_string()
            ));
        }

        tracing::debug!(
            pr = payload.number,
            reviewer = %reviewer_login,
            "Sending review-requested DM"
        );

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id: social_account.social_chat_id,
                message,
            })
            .await
            .ok();
    }
}

impl WebhookReviewRequestedDmListener {
    async fn handle_reviewer_on_vacation(
        &self,
        payload: &WebhookPullRequestEvent,
        reviewer_login: &str,
        until: chrono::DateTime<Utc>,
    ) {
        let (owner, name) = match payload.repo.split_once('/') {
            Some(parts) => parts,
            None => {
                tracing::warn!(repo = %payload.repo, "Cannot parse repo as owner/name");
                return;
            }
        };

        let comment_body = format!(
            "❄️ @{} в отпуске до {}. Пожалуйста, переназначьте ревью на другого.",
            reviewer_login,
            until.format("%d.%m.%Y")
        );

        if let Some(token) = self.author_access_token(&payload.author).await {
            if let Err(e) = self
                .version_control_client
                .post_pr_comment(&token, owner, name, payload.number, &comment_body)
                .await
            {
                tracing::warn!(error = %e, pr = payload.number, "Failed to post vacation comment in PR");
            }
        } else {
            tracing::debug!(
                author = %payload.author,
                "PR author has no token — skipping vacation comment in PR"
            );
        }

        self.notify_admins(payload, reviewer_login, until).await;
    }

    async fn author_access_token(&self, author_login: &str) -> Option<String> {
        let vc = self
            .user_vc_accounts_repo
            .find_by_login(author_login)
            .await
            .ok()?;
        match self.reversible_cipher.decrypt(vc.access_token.value()) {
            Ok(token) => Some(token),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to decrypt author access token");
                None
            }
        }
    }

    async fn notify_admins(
        &self,
        payload: &WebhookPullRequestEvent,
        reviewer_login: &str,
        until: chrono::DateTime<Utc>,
    ) {
        let admin_ids = match self
            .user_has_roles_repo
            .find_user_ids_by_role(RoleName::Admin)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to load admins for vacation alert");
                return;
            }
        };

        let pr_url = payload.pr_url.as_deref().unwrap_or("");
        let mut message = MessageBuilder::new()
            .bold(&t!("telegram_bot.notifications.vacation_review_alert.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.vacation_review_alert.reviewer").to_string(),
                reviewer_login,
            )
            .section(
                &t!("telegram_bot.notifications.vacation_review_alert.until").to_string(),
                &until.format("%d.%m.%Y").to_string(),
            )
            .section(
                &t!("telegram_bot.notifications.vacation_review_alert.pr").to_string(),
                &format!("#{} — {}", payload.number, payload.title),
            )
            .section(
                &t!("telegram_bot.notifications.vacation_review_alert.repository").to_string(),
                &payload.repo,
            )
            .with_html_escape(false);

        if !pr_url.is_empty() {
            message = message.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url),
                t!("telegram_bot.notifications.vacation_review_alert.open").to_string()
            ));
        }

        for user_id in &admin_ids {
            let social = match self.user_socials_repo.find_by_user_id(user_id).await {
                Ok(s) => s,
                Err(_) => continue,
            };
            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id: social.social_chat_id,
                    message: message.clone(),
                })
                .await
                .ok();
        }
    }
}
