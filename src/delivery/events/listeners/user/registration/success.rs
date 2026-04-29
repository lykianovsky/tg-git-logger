use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::events::registration_success::UserRegistrationSuccessEvent;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use rust_i18n::t;
use std::sync::Arc;

pub struct UserRegistrationSuccessListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub telegram_admin_user_id: SocialUserId,
}

#[async_trait]
impl EventListener<UserRegistrationSuccessEvent> for UserRegistrationSuccessListener {
    async fn handle(&self, payload: &UserRegistrationSuccessEvent) {
        let mut message = MessageBuilder::new()
            .line(&t!("notifications.registration.success_title").to_string())
            .empty_line()
            .line(
                &t!(
                    "notifications.registration.success_user_id",
                    id = payload.user.id.0
                )
                .to_string(),
            )
            .line(
                &t!(
                    "notifications.registration.success_github",
                    login = payload.user_version_control_account.version_control_login
                )
                .to_string(),
            )
            .line(
                &t!(
                    "notifications.registration.success_social",
                    social_type = format!("{:?}", payload.social_type)
                )
                .to_string(),
            )
            .line(
                &t!(
                    "notifications.registration.success_chat_id",
                    chat_id = payload.chat_id.0
                )
                .to_string(),
            )
            .empty_line()
            .line(&t!("notifications.registration.success_ready").to_string())
            .empty_line()
            .line(&t!("telegram_bot.notifications.registration.next_step").to_string());

        tracing::debug!(
            "{}, {}",
            payload.user_social_account.id,
            self.telegram_admin_user_id.0
        );
        if payload.user_social_account.social_user_id == self.telegram_admin_user_id {
            message = message
                .empty_line()
                .line(&t!("notifications.registration.success_admin_greeting").to_string())
        }

        self.publisher
            .publish(&SendSocialNotifyJob {
                social_type: payload.social_type,
                chat_id: payload.chat_id,
                message,
            })
            .await
            .ok();
    }
}
