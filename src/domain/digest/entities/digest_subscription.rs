use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;
use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSubscription {
    pub id: DigestSubscriptionId,
    pub user_id: UserId,
    pub repository_id: Option<RepositoryId>,
    pub digest_type: DigestType,
    pub send_hour: i8,
    pub send_minute: i8,
    pub day_of_week: Option<i8>,
    pub is_active: bool,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
