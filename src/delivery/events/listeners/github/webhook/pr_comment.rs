use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::events::pr_comment::WebhookPrCommentEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct WebhookPrCommentEventListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

#[async_trait]
impl EventListener<WebhookPrCommentEvent> for WebhookPrCommentEventListener {
    async fn handle(&self, payload: &WebhookPrCommentEvent) {
        // Don't notify the PR author when they comment on their own PR.
        if payload.commenter.eq_ignore_ascii_case(&payload.pr_author) {
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
                    "No VC account found for PR author — skipping comment notification"
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
                    "No social account found for PR author — skipping comment notification"
                );
                return;
            }
        };

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
