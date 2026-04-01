use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;

pub struct UpdateHealthPingCommand {
    pub id: HealthPingId,
    pub name: Option<String>,
    pub url: Option<String>,
    pub interval_minutes: Option<i32>,
    pub is_active: Option<bool>,
}
