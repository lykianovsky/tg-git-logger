use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::user_id::UserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::{DateTime, Utc};

pub struct BufferNotificationExecutorCommand {
    /// `None` для group chat (без привязки к конкретному юзеру).
    pub user_id: Option<UserId>,
    pub social_type: SocialType,
    pub chat_id: SocialChatId,
    pub message: MessageBuilder,
    pub event_type: String,
    pub deliver_after: DateTime<Utc>,
}
