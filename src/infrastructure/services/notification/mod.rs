use crate::domain::notification::services::notification_service::{
    NotificationService, NotificationServiceSendError,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::services::notification::telegram::TelegramNotificationClient;

pub mod telegram;

pub struct CompositionNotificationService {
    telegram: TelegramNotificationClient,
}

impl CompositionNotificationService {
    pub fn new(telegram_token: String) -> Self {
        Self {
            telegram: TelegramNotificationClient::new(telegram_token),
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for CompositionNotificationService {
    async fn send(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &str,
    ) -> Result<(), NotificationServiceSendError> {
        match social_type {
            SocialType::Telegram => self.telegram.send(social_type, chat_id, message).await,
        }
    }
}
