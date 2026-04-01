use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPing {
    pub id: HealthPingId,
    pub name: String,
    pub url: String,
    pub interval_minutes: i32,
    pub is_active: bool,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_response_ms: Option<i32>,
    pub last_error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
