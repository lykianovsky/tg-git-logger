use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use crate::domain::user::value_objects::version_control_user_id::VersionControlUserId;
use crate::utils::security::crypto::reversible::ReversibleCipherValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserVersionControlAccount {
    pub id: i32,
    pub user_id: UserId,
    pub version_control_type: VersionControlType,
    pub version_control_user_id: VersionControlUserId,
    pub version_control_login: String,
    pub version_control_email: Option<String>,
    pub version_control_avatar_url: Option<String>,
    pub access_token: ReversibleCipherValue,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
