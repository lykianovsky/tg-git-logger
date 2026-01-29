use crate::utils::notifier::Notifier;

use crate::client::telegram::TELEGRAM_BOT;
use crate::utils::notifier::message_builder::MessageBuilder;
use async_trait::async_trait;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{ParseMode, Recipient};

pub struct TelegramNotifierAdapter {
    default_chat_id: i64,
}

impl TelegramNotifierAdapter {
    pub fn new(default_chat_id: i64) -> Self {
        Self { default_chat_id }
    }
}

#[async_trait]
impl Notifier for TelegramNotifierAdapter {
    async fn send_message(&self, chat_id: i64, text: &MessageBuilder) -> Result<(), String> {
        let _ = TELEGRAM_BOT
            .send_message(
                Recipient::Id(teloxide::prelude::ChatId(chat_id)),
                text.to_string(),
            )
            .parse_mode(ParseMode::Html)
            .await;

        Ok(())
    }

    async fn notify(&self, text: &MessageBuilder) -> Result<(), String> {
        self.send_message(self.default_chat_id, text).await
    }
}
