use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::events::registration_failed::UserRegistrationFailedEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use rust_i18n::t;
use std::sync::Arc;

pub struct UserRegistrationFailedListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

#[async_trait]
impl EventListener<UserRegistrationFailedEvent> for UserRegistrationFailedListener {
    async fn handle(&self, payload: &UserRegistrationFailedEvent) {
        let message = MessageBuilder::new()
            .line(&t!("notifications.registration.failed_title").to_string())
            .empty_line()
            .line(&t!("notifications.registration.failed_body").to_string())
            .empty_line()
            .empty_line()
            .line(&t!("notifications.registration.failed_retry").to_string())
            .line(&t!("notifications.registration.failed_support").to_string());

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
