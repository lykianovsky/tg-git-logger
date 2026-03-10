use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageBrokerQueueRetryPolicy {
    pub max_attempts: i64,
    pub delay_ms: i64,
}

impl Default for MessageBrokerQueueRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: Duration::from_secs(30).as_millis() as i64,
        }
    }
}

pub struct MessageBrokerQueue {
    pub name: String,
    pub routing_key: String,
    pub retry_policy: Option<MessageBrokerQueueRetryPolicy>,
}
