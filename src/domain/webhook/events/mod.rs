use crate::domain::shared::events::event::StaticDomainEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerMessage;

pub mod pr_comment;
pub mod pull_request;
pub mod pull_request_review;
pub mod push;
pub mod release;
pub mod workflow;

pub trait WebhookEvent: StaticDomainEvent + MessageBrokerMessage + Send + Sync {
    fn build_text(&self) -> String;
}
