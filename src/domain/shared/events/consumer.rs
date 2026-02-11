use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::shared::events::retry_policy::RetryPolicy;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventConsumerError {
    #[error("Handler failed: {0}")]
    HandlerFailed(String),
    #[error("Failed to deserialize payload: {0}")]
    DeserializationFailed(String),
}

#[async_trait]
pub trait EventConsumer: Send + Sync {
    type EventPayload: StaticDomainEvent + DeserializeOwned + Send;

    fn routing_key(&self) -> &'static str;

    fn queue_name(&self) -> &'static str;

    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::default()
    }

    async fn handle(&self, event_payload: Self::EventPayload) -> Result<(), EventConsumerError>;
}
