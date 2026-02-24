#[derive(Clone)]
pub struct MessageBrokerQueueRetryPolicy {
    pub max_attempts: i64,
    pub delay_ms: u32,
}

impl Default for MessageBrokerQueueRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: 5000,
        }
    }
}

pub struct MessageBrokerQueue {
    pub name: String,
    pub routing_key: String,
    pub retry_policy: Option<MessageBrokerQueueRetryPolicy>,
}
