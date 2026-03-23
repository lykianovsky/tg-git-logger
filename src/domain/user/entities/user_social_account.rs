use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSocialAccount {
    pub id: i32,
    pub user_id: UserId,
    pub social_type: SocialType,
    pub social_user_id: SocialUserId,
    pub social_chat_id: SocialChatId,
    pub social_user_login: Option<String>,
    pub social_user_email: Option<String>,
    pub social_user_avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
