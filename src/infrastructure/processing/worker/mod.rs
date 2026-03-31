use async_trait::async_trait;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Error, Debug)]
pub enum MessageBrokerWorkerStartError {
    #[error("Failed to create consumer stream: {0}")]
    FailedToCreateConsumerStream(String),
}

#[async_trait]
pub trait MessageBrokerWorker: Send + Sync + 'static {
    fn name(&self) -> &str;

    async fn start(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Result<(), MessageBrokerWorkerStartError>;
}
