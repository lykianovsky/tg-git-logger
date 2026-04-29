use crate::delivery::events::listeners::github::webhook::resolve_notifications_chat_id;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::pr_review::repositories::pr_review_repository::PrReviewRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::events::pull_request_review::{
    WebhookPullRequestReviewEvent, WebhookPullRequestReviewState,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

pub struct WebhookPullRequestReviewEventListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub pr_review_repo: Arc<dyn PrReviewRepository>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookPullRequestReviewEvent> for WebhookPullRequestReviewEventListener {
    async fn handle(&self, payload: &WebhookPullRequestReviewEvent) {
        let state_str = match payload.state {
            WebhookPullRequestReviewState::Approved => "approved",
            WebhookPullRequestReviewState::ChangesRequested => "changes_requested",
            WebhookPullRequestReviewState::Commented => "commented",
            WebhookPullRequestReviewState::Unknown => "unknown",
        };

        if let Err(e) = self
            .pr_review_repo
            .upsert(
                &payload.repo,
                payload.pr_number,
                &payload.reviewer,
                state_str,
                Utc::now(),
            )
            .await
        {
            tracing::warn!(
                error = %e,
                repo = %payload.repo,
                pr = payload.pr_number,
                reviewer = %payload.reviewer,
                "Failed to upsert pr_review"
            );
        }

        // Короткое сообщение в групповой чат для approved/changes_requested
        if matches!(
            payload.state,
            WebhookPullRequestReviewState::Approved
                | WebhookPullRequestReviewState::ChangesRequested
        ) {
            self.post_group_chat_short(payload).await;
        }

        if payload.reviewer.eq_ignore_ascii_case(&payload.pr_author) {
            return;
        }

        let vc_account = match self
            .user_vc_accounts_repo
            .find_by_login(&payload.pr_author)
            .await
        {
            Ok(account) => account,
            Err(_) => {
                tracing::debug!(
                    pr_author = %payload.pr_author,
                    "No VC account found for PR author — skipping review notification"
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
                    "No social account found for PR author — skipping review notification"
                );
                return;
            }
        };

        tracing::debug!(
            pr_author = %payload.pr_author,
            reviewer = %payload.reviewer,
            state = ?payload.state,
            "Sending PR review DM notification"
        );

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id: social_account.social_chat_id,
                message: MessageBuilder::new().raw(payload.build_text().as_str()),
            })
            .await
            .ok();
    }
}

impl WebhookPullRequestReviewEventListener {
    async fn post_group_chat_short(&self, payload: &WebhookPullRequestReviewEvent) {
        let chat_id = resolve_notifications_chat_id(
            &self.repository_repo,
            &payload.repo,
            self.default_chat_id,
        )
        .await;

        let reviewer_tag = match self.resolve_tg_username(&payload.reviewer).await {
            Some(tg) => format!("@{}", tg),
            None => format!("@{}", payload.reviewer),
        };

        let prefix = match payload.state {
            WebhookPullRequestReviewState::Approved => {
                t!("telegram_bot.notifications.review_chat.approved").to_string()
            }
            WebhookPullRequestReviewState::ChangesRequested => {
                t!("telegram_bot.notifications.review_chat.changes_requested").to_string()
            }
            _ => return,
        };

        let line = format!(
            "{} <a href=\"{}\">#{}</a> — {}",
            prefix, payload.pr_url, payload.pr_number, reviewer_tag
        );
        let msg = MessageBuilder::new().raw(&line);

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id,
                message: msg,
            })
            .await
            .ok();
    }

    async fn resolve_tg_username(&self, github_login: &str) -> Option<String> {
        let vc = self
            .user_vc_accounts_repo
            .find_by_login(github_login)
            .await
            .ok()?;
        let social = self
            .user_socials_repo
            .find_by_user_id(&vc.user_id)
            .await
            .ok()?;
        social.social_user_login
    }
}
