use async_trait::async_trait;
use thiserror::Error;

pub enum JobConsumerResponse {
    Ok,
    Retry,
    Reject(String),
    Requeue,
}

#[derive(Error, Debug)]
pub enum JobConsumerError {
    #[error("Failed to deserialization: {0}")]
    DeserializationError(String),
}

#[async_trait]
pub trait JobConsumer: Send + Sync {
    fn name(&self) -> &'static str;

    async fn run(&self, payload: &[u8]) -> Result<JobConsumerResponse, JobConsumerError>;
}
