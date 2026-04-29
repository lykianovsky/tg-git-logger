use crate::delivery::events::listeners::github::webhook::resolve_notifications_chat_id;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::pull_request::{
    WebhookPullRequestEvent, WebhookPullRequestEventActionType,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPrOpenedTagReviewersListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPrOpenedTagReviewersListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        if !matches!(
            payload.action,
            WebhookPullRequestEventActionType::Opened
                | WebhookPullRequestEventActionType::ReadyForReview
        ) {
            return;
        }

        let reviewers_label = if payload.requested_reviewers.is_empty() {
            t!("telegram_bot.notifications.pr_opened_tag.no_reviewers").to_string()
        } else {
            let mut tag_parts: Vec<String> = Vec::new();
            for login in &payload.requested_reviewers {
                let display = match self.resolve_tg_username(login).await {
                    Some(tg_username) => format!("@{}", tg_username),
                    None => login.clone(),
                };
                tag_parts.push(display);
            }
            tag_parts.join(", ")
        };

        let chat_id = resolve_notifications_chat_id(
            &self.repository_repo,
            &payload.repo,
            self.default_chat_id,
        )
        .await;

        let pr_url = payload.pr_url.as_deref().unwrap_or("");
        let mut msg = MessageBuilder::new()
            .bold(&t!("telegram_bot.notifications.pr_opened_tag.title").to_string())
            .empty_line()
            .with_html_escape(true)
            .section(
                &t!("telegram_bot.notifications.pr_opened_tag.pr").to_string(),
                &format!("#{} — {}", payload.number, payload.title),
            )
            .section(
                &t!("telegram_bot.notifications.pr_opened_tag.author").to_string(),
                &payload.author,
            )
            .section(
                &t!("telegram_bot.notifications.pr_opened_tag.repository").to_string(),
                &payload.repo,
            )
            .section(
                &t!("telegram_bot.notifications.pr_opened_tag.reviewers").to_string(),
                &reviewers_label,
            );

        if !pr_url.is_empty() {
            msg = msg.empty_line().raw(&format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url),
                t!("telegram_bot.notifications.pr_opened_tag.open").to_string()
            ));
        }

        tracing::debug!(
            pr = payload.number,
            reviewers = ?payload.requested_reviewers,
            "Posting opened-PR tag-message in group chat"
        );

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: SocialType::Telegram,
                chat_id,
                message: msg,
            })
            .await
            .ok();
    }
}

impl WebhookPrOpenedTagReviewersListener {
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
