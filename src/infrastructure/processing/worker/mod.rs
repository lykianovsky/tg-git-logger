use async_trait::async_trait;

pub enum MessageBrokerWorkerStartError {
    Test,
}

#[async_trait]
pub trait MessageBrokerWorker: Send + 'static {
    async fn start(self) -> Result<(), MessageBrokerWorkerStartError>;
}
