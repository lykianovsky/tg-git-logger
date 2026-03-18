use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageBrokerWorkerStartError {
    #[error("Failed to create consumer stream: {0}")]
    FailedToCreateConsumerStream(String),
}

#[async_trait]
pub trait MessageBrokerWorker: Send + 'static {
    fn name(&self) -> &str;

    async fn start(self) -> Result<(), MessageBrokerWorkerStartError>;
}
