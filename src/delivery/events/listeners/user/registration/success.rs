use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::events::event_listener::EventListener;
use crate::domain::user::events::registration_success::UserRegistrationSuccessEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct UserRegistrationSuccessListener {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

#[async_trait]
impl EventListener<UserRegistrationSuccessEvent> for UserRegistrationSuccessListener {
    async fn handle(&self, payload: &UserRegistrationSuccessEvent) {
        let message = MessageBuilder::new()
            .line("✅ Регистрация успешно завершена!")
            .empty_line()
            .line(&format!("👤 Пользователь (user_id): {}", payload.user.id.0))
            .line(&format!(
                "🔗 GitHub: {}",
                payload.user_version_control_account.version_control_login
            ))
            .line(&format!("📱 Соц. сеть: {:?}", payload.social_type))
            .line(&format!("🆔 Chat ID: {}", payload.chat_id.0))
            .empty_line()
            .line("🚀 Теперь можно пользоваться ботом!");

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
