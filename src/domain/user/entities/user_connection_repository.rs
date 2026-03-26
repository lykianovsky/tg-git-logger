use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnectionRepository {
    pub id: i32,
    pub user_id: UserId,
    pub repository_id: RepositoryId,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
