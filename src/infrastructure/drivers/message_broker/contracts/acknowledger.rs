use async_trait::async_trait;

#[async_trait]
pub trait BrokerMessageAcknowledger: Send + Sync {
    async fn ack(&self);
    async fn nack(&self, requeue: bool);
    async fn reject(&self, reason: &str); // сразу в dead
}
