use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub is_active: bool,
    pub create_at: DateTime<Utc>,
    pub update_at: DateTime<Utc>,
}
