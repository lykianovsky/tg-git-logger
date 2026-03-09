use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::utils::builder::message::MessageBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationServiceSendError {
    #[error("{0}")]
    UnsupportedSocialType(String),
    #[error("{0}")]
    Transport(String),
}

pub enum NotificationServiceParseMode {
    Html,
    Markdown,
}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceSendError>;
}
