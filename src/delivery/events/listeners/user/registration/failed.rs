use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::events::registration_failed::UserRegistrationFailedEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct UserRegistrationFailedListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

#[async_trait]
impl EventListener<UserRegistrationFailedEvent> for UserRegistrationFailedListener {
    async fn handle(&self, payload: &UserRegistrationFailedEvent) {
        let message = MessageBuilder::new()
            .line("❌ Ошибка регистрации")
            .empty_line()
            .line("Не удалось завершить авторизацию через OAuth.")
            .empty_line()
            .empty_line()
            .line("🔁 Попробуйте ещё раз.")
            .line("Если проблема повторяется — обратитесь в поддержку.");

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
