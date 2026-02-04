use crate::client::telegram::TELEGRAM_BOT;
use crate::domain::notification::service::{NotificationSendToChatError, NotificationService};
use crate::utils::builder::message::MessageBuilder;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{ParseMode, Recipient};

pub struct TelegramNotifierService {
    token: String
}

impl TelegramNotifierService {
    pub fn new(token: String) -> Self {
        Self {
            token
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for TelegramNotifierService {
    async fn send_to_chat(&self, chat_id: i64, text: &MessageBuilder) -> Result<(), NotificationSendToChatError> {
        let _ = TELEGRAM_BOT
            .send_message(
                Recipient::Id(teloxide::prelude::ChatId(chat_id)),
                text.to_string(),
            )
            .parse_mode(ParseMode::Html)
            .await;

        Ok(())
    }
}