use crate::domain::notification::services::notification_service::{
    NotificationService, NotificationServiceDeleteMessageError,
    NotificationServiceEditMessageError, NotificationServiceSendError,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::utils::builder::message::MessageBuilder;
use teloxide::prelude::*;
use teloxide::types::{ChatId, MessageId, ParseMode};

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
    async fn send_message(
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

        let started_at = std::time::Instant::now();

        let result = self
            .bot
            .send_message(ChatId(chat_id.0), message.to_string())
            .parse_mode(ParseMode::Html)
            .await;

        let elapsed = started_at.elapsed().as_secs_f64();

        match result {
            Ok(_) => {
                crate::infrastructure::metrics::registry::METRICS
                    .telegram_send_total
                    .with_label_values(&["success"])
                    .inc();
                crate::infrastructure::metrics::registry::METRICS
                    .telegram_send_duration_seconds
                    .with_label_values(&["success"])
                    .observe(elapsed);
                tracing::debug!(chat_id = chat_id.0, "Telegram notification sent");
                Ok(())
            }
            Err(e) => {
                crate::infrastructure::metrics::registry::METRICS
                    .telegram_send_total
                    .with_label_values(&["fail"])
                    .inc();
                crate::infrastructure::metrics::registry::METRICS
                    .telegram_send_duration_seconds
                    .with_label_values(&["fail"])
                    .observe(elapsed);
                tracing::error!(
                    error = %e,
                    chat_id = chat_id.0,
                    "Failed to send Telegram notification"
                );
                Err(NotificationServiceSendError::Transport(e.to_string()))
            }
        }
    }

    async fn delete_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
    ) -> Result<(), NotificationServiceDeleteMessageError> {
        if *social_type != SocialType::Telegram {
            return Err(
                NotificationServiceDeleteMessageError::UnsupportedSocialType(
                    social_type.to_string(),
                ),
            );
        }

        self.bot
            .delete_message(ChatId(chat_id.0), MessageId(message_id.0))
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    chat_id = chat_id.0,
                    message_id = message_id.0,
                    "Failed to delete message Telegram notification"
                );
                NotificationServiceDeleteMessageError::Transport(e.to_string())
            })?;

        Ok(())
    }

    async fn edit_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceEditMessageError> {
        if *social_type != SocialType::Telegram {
            return Err(NotificationServiceEditMessageError::UnsupportedSocialType(
                social_type.to_string(),
            ));
        }

        self.bot
            .edit_message_text(
                ChatId(chat_id.0),
                MessageId(message_id.0),
                message.to_string(),
            )
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    chat_id = chat_id.0,
                    message_id = message_id.0,
                    "Failed to delete message Telegram notification"
                );
                NotificationServiceEditMessageError::Transport(e.to_string())
            })?;

        Ok(())
    }
}
