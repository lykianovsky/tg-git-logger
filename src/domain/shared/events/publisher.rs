use crate::domain::shared::events::event::DomainEvent;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventPublisherError {
    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), EventPublisherError>;
}
