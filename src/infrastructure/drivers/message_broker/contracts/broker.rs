use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueue;
use crate::infrastructure::drivers::message_broker::contracts::queue_builder::MessageBrokerStream;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageBrokerConsumeError {
    #[error("Failed to create channel: {0}")]
    ChannelCreate(String),

    #[error("Failed to create channel stream: {0}")]
    ChannelStreamCreate(String),
}

#[derive(Error, Debug)]
pub enum MessageBrokerSetupError {
    #[error("Message broker setup error: {0}")]
    SetupError(String),
}

#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn consume<'a>(
        &'a self,
        worker_name: &str,
        queue: &'a MessageBrokerQueue,
    ) -> Result<MessageBrokerStream<'a>, MessageBrokerConsumeError>;

    async fn setup(
        &self,
        queues: Vec<Arc<MessageBrokerQueue>>,
    ) -> Result<(), MessageBrokerSetupError>;
}
