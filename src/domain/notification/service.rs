use crate::utils::builder::message::MessageBuilder;
use std::fmt;

#[derive(Debug)]
pub enum NotificationSendToChatError {}

impl fmt::Display for NotificationSendToChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Notification Send To Chat Error")
    }
}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_to_chat(&self, chat_id: i64, text: &MessageBuilder) -> Result<(), NotificationSendToChatError>;
}