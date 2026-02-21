use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationServiceSendError {
    #[error("{0}")]
    UnsupportedSocialType(String),
    #[error("{0}")]
    Transport(String),
}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send(
        &self,
        social_type: &SocialType,
        chat_id: &SocialChatId,
        message: &str,
    ) -> Result<(), NotificationServiceSendError>;
}
