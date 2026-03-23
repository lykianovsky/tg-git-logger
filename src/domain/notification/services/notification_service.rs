use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
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

#[derive(Debug, Error)]
pub enum NotificationServiceDeleteMessageError {
    #[error("{0}")]
    UnsupportedSocialType(String),
    #[error("{0}")]
    Transport(String),
}

#[derive(Debug, Error)]
pub enum NotificationServiceEditMessageError {
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
    async fn send_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceSendError>;

    async fn delete_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
    ) -> Result<(), NotificationServiceDeleteMessageError>;

    async fn edit_message(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message_id: &SocialMessageId,
        message: &MessageBuilder,
    ) -> Result<(), NotificationServiceEditMessageError>;
}
