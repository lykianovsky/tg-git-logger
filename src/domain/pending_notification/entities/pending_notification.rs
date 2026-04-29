use crate::domain::pending_notification::value_objects::pending_notification_id::PendingNotificationId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::user_id::UserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PendingNotification {
    pub id: PendingNotificationId,
    /// `None` для уведомлений в групповой чат (без привязки к юзеру).
    pub user_id: Option<UserId>,
    pub social_type: SocialType,
    pub social_chat_id: SocialChatId,
    pub message: MessageBuilder,
    pub event_type: String,
    pub deliver_after: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
