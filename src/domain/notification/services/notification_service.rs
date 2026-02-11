use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;

pub enum NotificationServiceSendError {
    UnsupportedSocialType(String),
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
