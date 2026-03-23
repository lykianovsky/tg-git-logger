use crate::domain::shared::events::event::DomainEvent;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistrationFailedEvent {
    pub social_type: SocialType,
    pub chat_id: SocialChatId,
}

impl DomainEvent for UserRegistrationFailedEvent {
    const EVENT_NAME: &'static str = "user.registration.failed";
}

impl MessageBrokerMessage for UserRegistrationFailedEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
