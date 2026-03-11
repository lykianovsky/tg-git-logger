use async_trait::async_trait;

#[async_trait]
pub trait BrokerMessageAcknowledger: Send + Sync {
    async fn ack(&self);
    async fn requeue(&self);
    async fn retry(&self, reason: &str);
    async fn reject(&self, reason: &str); // сразу в dead
}
