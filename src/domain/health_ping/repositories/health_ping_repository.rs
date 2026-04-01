use crate::domain::health_ping::entities::health_ping::HealthPing;
use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateHealthPingError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindHealthPingError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Health ping not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateHealthPingError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Health ping not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DeleteHealthPingError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Health ping not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait HealthPingRepository: Send + Sync {
    async fn create(&self, ping: &HealthPing) -> Result<HealthPing, CreateHealthPingError>;

    async fn find_by_id(&self, id: HealthPingId) -> Result<HealthPing, FindHealthPingError>;

    async fn find_all(&self) -> Result<Vec<HealthPing>, FindHealthPingError>;

    async fn find_active_due(&self) -> Result<Vec<HealthPing>, FindHealthPingError>;

    async fn update(&self, ping: &HealthPing) -> Result<HealthPing, UpdateHealthPingError>;

    async fn delete(&self, id: HealthPingId) -> Result<(), DeleteHealthPingError>;
}
