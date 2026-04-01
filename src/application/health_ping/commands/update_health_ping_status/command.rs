use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use chrono::{DateTime, Utc};

pub struct UpdateHealthPingStatusCommand {
    pub id: HealthPingId,
    pub status: String,
    pub response_ms: Option<i32>,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}
