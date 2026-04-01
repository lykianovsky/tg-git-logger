use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;

pub struct DeleteHealthPingCommand {
    pub id: HealthPingId,
}
