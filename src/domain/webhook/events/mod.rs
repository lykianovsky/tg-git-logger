use crate::domain::shared::events::event::StaticDomainEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerMessage;

pub mod pull_request;
pub mod push;
pub mod release;
pub mod workflow;

pub trait WebhookEvent: StaticDomainEvent + MessageBrokerMessage + Send + Sync {
    fn build_text(&self) -> String;
}
