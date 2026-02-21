use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;

pub struct NotifyReceivedWebhookEventExecutorCommand {
    pub social_type: SocialType,
    pub chat_id: SocialChatId,
    pub message: String,
}
