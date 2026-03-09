use crate::domain::shared::events::event::DomainEvent;
use async_trait::async_trait;

#[async_trait]
pub trait EventListener<E: DomainEvent>: Send + Sync {
    async fn handle(&self, payload: &E);
}
