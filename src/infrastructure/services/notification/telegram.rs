use crate::domain::notification::services::notification_service::{
    NotificationService, NotificationServiceSendError,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::utils::builder::message::MessageBuilder;
use teloxide::prelude::*;
use teloxide::types::{ChatId, ParseMode};

pub struct TelegramNotificationClient {
    bot: Bot,
}

impl TelegramNotificationClient {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot: Bot::new(bot_token),
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for TelegramNotificationClient {
    async fn send(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceSendError> {
        if *social_type != SocialType::Telegram {
            return Err(NotificationServiceSendError::UnsupportedSocialType(
                social_type.to_string(),
            ));
        }

        self.bot
            .send_message(ChatId(chat_id.0), message.to_string())
            .parse_mode(ParseMode::Html)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    chat_id = chat_id.0,
                    "Failed to send Telegram notification"
                );
                NotificationServiceSendError::Transport(e.to_string())
            })?;

        tracing::debug!(chat_id = chat_id.0, "Telegram notification sent");

        Ok(())
    }
}
