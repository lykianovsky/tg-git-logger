// bootstrap/queues.rs
use crate::infrastructure::drivers::message_broker::contracts::queue::{
    MessageBrokerQueue, MessageBrokerQueueRetryPolicy,
};
use std::fmt;

pub enum ApplicationQueueName {
    Events,
    JobsCritical,
    JobsNormal,
    JobsBackground,
}

impl fmt::Display for ApplicationQueueName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            ApplicationQueueName::Events => "events",
            ApplicationQueueName::JobsCritical => "jobs.critical",
            ApplicationQueueName::JobsNormal => "jobs.normal",
            ApplicationQueueName::JobsBackground => "jobs.background",
        };

        write!(f, "{}", str)
    }
}

use std::sync::Arc;
use std::time::Duration;

pub struct ApplicationQueues {
    pub events: Arc<MessageBrokerQueue>,
    pub jobs_critical: Arc<MessageBrokerQueue>,
    pub jobs_normal: Arc<MessageBrokerQueue>,
    pub jobs_background: Arc<MessageBrokerQueue>,
}

impl ApplicationQueues {
    pub fn new() -> Self {
        Self {
            events: Arc::new(MessageBrokerQueue {
                name: ApplicationQueueName::Events.to_string(),
                routing_key: ApplicationQueueName::Events.to_string(),
                retry_policy: None,
            }),
            jobs_critical: Arc::new(MessageBrokerQueue {
                name: ApplicationQueueName::JobsCritical.to_string(),
                routing_key: ApplicationQueueName::JobsCritical.to_string(),
                retry_policy: Some(MessageBrokerQueueRetryPolicy {
                    max_attempts: 5,
                    delay_ms: Duration::from_secs(5).as_millis() as i64,
                }),
            }),
            jobs_normal: Arc::new(MessageBrokerQueue {
                name: ApplicationQueueName::JobsNormal.to_string(),
                routing_key: ApplicationQueueName::JobsNormal.to_string(),
                retry_policy: Some(MessageBrokerQueueRetryPolicy {
                    max_attempts: 3,
                    delay_ms: Duration::from_secs(30).as_millis() as i64,
                }),
            }),
            jobs_background: Arc::new(MessageBrokerQueue {
                name: ApplicationQueueName::JobsBackground.to_string(),
                routing_key: ApplicationQueueName::JobsBackground.to_string(),
                retry_policy: Some(MessageBrokerQueueRetryPolicy {
                    max_attempts: 2,
                    delay_ms: Duration::from_secs(60).as_millis() as i64,
                }),
            }),
        }
    }
}
