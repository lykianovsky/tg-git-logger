use crate::client::telegram::bot::TelegramBot;
use crate::utils::notifier::Notifier;

use crate::utils::notifier::message_builder::MessageBuilder;
use async_trait::async_trait;
use std::sync::Arc;

pub struct TelegramNotifierAdapter {
    bot: Arc<dyn TelegramBot>,
    default_chat_id: i64,
}

impl TelegramNotifierAdapter {
    pub fn new(bot: Arc<dyn TelegramBot>, default_chat_id: i64) -> Self {
        Self {
            bot,
            default_chat_id,
        }
    }
}

#[async_trait]
impl Notifier for TelegramNotifierAdapter {
    async fn send_message(&self, chat_id: i64, text: &MessageBuilder) -> Result<(), String> {
        self.bot
            .send_message(chat_id, text.to_string().as_str())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn notify(&self, text: &MessageBuilder) -> Result<(), String> {
        self.send_message(self.default_chat_id, text).await
    }
}
