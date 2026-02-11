use crate::domain::shared::events::consumer::EventConsumer;
use async_trait::async_trait;

#[async_trait]
pub trait EventConsumerRunner: Sized {
    fn register<C: EventConsumer + Send + Sync + 'static>(self, consumer: C) -> Self;

    async fn run(self) -> Result<(), String>;
}
