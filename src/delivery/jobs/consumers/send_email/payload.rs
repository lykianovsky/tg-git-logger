use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendEmailJob {
    pub(crate) email: String,
}

impl SendEmailJob {
    pub const NAME: &'static str = "send_email";
}

impl MessageBrokerMessage for SendEmailJob {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Job
    }
}
