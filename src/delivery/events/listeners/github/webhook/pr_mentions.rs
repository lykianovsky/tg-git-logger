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
use crate::utils::parsing::mentions::extract_github_mentions;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPrMentionsListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub default_chat_id: SocialChatId,
}

#[async_trait]
impl EventListener<WebhookPullRequestEvent> for WebhookPrMentionsListener {
    async fn handle(&self, payload: &WebhookPullRequestEvent) {
        if !matches!(
            payload.action,
            WebhookPullRequestEventActionType::Opened
                | WebhookPullRequestEventActionType::ReadyForReview
        ) {
            return;
        }

        let mut combined = payload.title.clone();
        if let Some(body) = &payload.body {
            combined.push(' ');
            combined.push_str(body);
        }

        let mentions = extract_github_mentions(&combined);
        if mentions.is_empty() {
            return;
        }

        let mut bound: Vec<(String, SocialChatId, Option<String>)> = Vec::new();

        for login in &mentions {
            if login.eq_ignore_ascii_case(&payload.author) {
                continue;
            }
            let vc_account = match self.user_vc_accounts_repo.find_by_login(login).await {
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
            bound.push((
                login.clone(),
                social_account.social_chat_id,
                social_account.social_user_login,
            ));
        }

        if bound.is_empty() {
            return;
        }

        let pr_url = payload.pr_url.as_deref().unwrap_or("");

        for (login, chat_id, _) in &bound {
            let mut dm = MessageBuilder::new()
                .bold(&t!("telegram_bot.notifications.pr_mention.title").to_string())
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.pr_mention.pr").to_string(),
                    &format!("#{} — {}", payload.number, payload.title),
                )
                .section(
                    &t!("telegram_bot.notifications.pr_mention.author").to_string(),
                    &payload.author,
                )
                .section(
                    &t!("telegram_bot.notifications.pr_mention.repository").to_string(),
                    &payload.repo,
                )
                .with_html_escape(false);

            if !pr_url.is_empty() {
                dm = dm.empty_line().raw(&format!(
                    "<a href=\"{}\">{}</a>",
                    pr_url,
                    t!("telegram_bot.notifications.pr_mention.open").to_string()
                ));
            }

            tracing::debug!(
                pr = payload.number,
                login = %login,
                "Sending PR mention DM"
            );

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id: *chat_id,
                    message: dm,
                })
                .await
                .ok();
        }

        let tags: Vec<String> = bound
            .iter()
            .filter_map(|(_, _, tg)| tg.as_ref().map(|n| format!("@{}", n)))
            .collect();

        if !tags.is_empty() {
            let chat_id = resolve_notifications_chat_id(
                &self.repository_repo,
                &payload.repo,
                self.default_chat_id,
            )
            .await;

            let mut msg = MessageBuilder::new()
                .raw(&format!("👀 cc: {}", tags.join(" ")))
                .empty_line()
                .with_html_escape(true)
                .raw(&format!("PR #{} — {}", payload.number, payload.title))
                .with_html_escape(false);

            if !pr_url.is_empty() {
                msg = msg.empty_line().raw(&format!(
                    "<a href=\"{}\">{}</a>",
                    pr_url,
                    t!("telegram_bot.notifications.pr_mention.open").to_string()
                ));
            }

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
}
