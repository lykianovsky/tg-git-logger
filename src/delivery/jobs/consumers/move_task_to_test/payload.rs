use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind, MessageBrokerMessageKindJobPriority,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MoveTaskToTestJob {
    pub task_id: u64,
}

impl MoveTaskToTestJob {
    pub const NAME: &'static str = "move_task_to_test";
}

impl MessageBrokerMessage for MoveTaskToTestJob {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Job(MessageBrokerMessageKindJobPriority::Critical)
    }
}
