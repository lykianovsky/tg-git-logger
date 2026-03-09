// bootstrap/queues.rs
use crate::infrastructure::drivers::message_broker::contracts::queue::{
    MessageBrokerQueue, MessageBrokerQueueRetryPolicy,
};
use std::sync::Arc;

pub struct ApplicationQueues {
    pub events: Arc<MessageBrokerQueue>,
    pub jobs: Arc<MessageBrokerQueue>,
}

impl ApplicationQueues {
    pub fn new() -> Self {
        Self {
            events: Arc::new(MessageBrokerQueue {
                name: "events".to_string(),
                routing_key: "events".to_string(),
                retry_policy: None,
            }),
            jobs: Arc::new(MessageBrokerQueue {
                name: "jobs".to_string(),
                routing_key: "jobs".to_string(),
                retry_policy: Some(MessageBrokerQueueRetryPolicy::default()),
            }),
        }
    }
}
