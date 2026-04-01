use crate::domain::health_ping::entities::health_ping::HealthPing;

pub struct GetAllHealthPingsResponse {
    pub pings: Vec<HealthPing>,
}
