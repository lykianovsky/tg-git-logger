use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};

pub struct UserListItem {
    pub user_id: UserId,
    pub is_active: bool,
    pub social_login: Option<String>,
    pub social_user_id: Option<i32>,
    pub roles: Vec<RoleName>,
    pub created_at: DateTime<Utc>,
}

pub struct GetAllUsersResponse {
    pub users: Vec<UserListItem>,
}
