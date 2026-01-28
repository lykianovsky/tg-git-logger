pub mod message_builder;
pub mod telegram;

use crate::client::notifier::message_builder::MessageBuilder;
use async_trait::async_trait;

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send_message(&self, chat_id: i64, text: &MessageBuilder) -> Result<(), String>;

    async fn notify(&self, text: &MessageBuilder) -> Result<(), String>;
}
