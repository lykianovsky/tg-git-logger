#[async_trait::async_trait]
pub trait ApplicationDelivery: Send + Sync {
    async fn serve(&self) -> Result<(), Box<dyn std::error::Error>>;
}
