use crate::domain::user::value_objects::notification_type::NotificationType;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotification {
    pub id: i32,
    pub user_id: UserId,
    pub notification_type: NotificationType,
    pub interval_minutes: i32,
    pub is_active: bool,
    pub last_notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
