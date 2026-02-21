use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenAuthorizationState {
    pub version_control_type: VersionControlType,
    pub social_type: SocialType,
    pub social_user_id: SocialUserId,
    pub social_chat_id: SocialChatId,
    pub social_user_login: Option<String>,
    pub social_user_email: Option<String>,
    pub social_user_avatar_url: Option<String>,
}
