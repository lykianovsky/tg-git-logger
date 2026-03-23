use crate::domain::notification::services::notification_service::{
    NotificationService, NotificationServiceDeleteMessageError,
    NotificationServiceEditMessageError, NotificationServiceSendError,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::services::notification::telegram::TelegramNotificationClient;
use crate::utils::builder::message::MessageBuilder;

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
    async fn send_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceSendError> {
        match social_type {
            SocialType::Telegram => {
                self.telegram
                    .send_message(social_type, chat_id, message)
                    .await
            }
        }
    }

    async fn delete_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
    ) -> Result<(), NotificationServiceDeleteMessageError> {
        match social_type {
            SocialType::Telegram => {
                self.telegram
                    .delete_message(social_type, chat_id, message_id)
                    .await
            }
        }
    }

    async fn edit_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceEditMessageError> {
        match social_type {
            SocialType::Telegram => {
                self.telegram
                    .edit_message(social_type, chat_id, message_id, message)
                    .await
            }
        }
    }
}
