use crate::domain::shared::events::event::{DomainEvent, StaticDomainEvent};

pub mod pull_request;
pub mod push;
pub mod release;
pub mod workflow;

pub trait WebhookEvent: DomainEvent + Send + Sync {
    fn build_text(&self) -> String;
}
