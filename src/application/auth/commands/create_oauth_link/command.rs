use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_message_id::SocialMessageId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;

#[derive(Debug, Clone)]
pub struct CreateOAuthLinkExecutorCommandSocial {
    pub r#type: SocialType,
    pub chat_id: SocialChatId,
    pub message_id: SocialMessageId,
    pub user_id: SocialUserId,
    pub user_login: Option<String>,
    pub user_email: Option<String>,
    pub user_avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateOAuthLinkExecutorCommandVersionControl {
    pub base: String,
    pub path: String,
    pub client_id: String,
    pub scope: String,
    pub r#type: VersionControlType,
}

#[derive(Debug, Clone)]
pub struct CreateOAuthLinkExecutorCommand {
    pub social: CreateOAuthLinkExecutorCommandSocial,
    pub version_control: CreateOAuthLinkExecutorCommandVersionControl,
    pub role: RoleName,
}
