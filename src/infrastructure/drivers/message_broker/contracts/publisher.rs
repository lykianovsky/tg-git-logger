use crate::bootstrap::queues::ApplicationQueueName;
use async_trait::async_trait;
use erased_serde::Serialize;
use std::fmt;

pub enum MessageBrokerPublisherPublishError {
    Serialization(String),
    PublishCreation(String),
    PublishConfirmation(String),
}

pub enum MessageBrokerMessageKindJobPriority {
    Critical,
    Normal,
    Background,
}

impl fmt::Display for MessageBrokerMessageKindJobPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            MessageBrokerMessageKindJobPriority::Critical => "critical",
            MessageBrokerMessageKindJobPriority::Normal => "normal",
            MessageBrokerMessageKindJobPriority::Background => "background",
        };

        write!(f, "{}", str)
    }
}

pub enum MessageBrokerMessageKind {
    Event,
    Job(MessageBrokerMessageKindJobPriority),
}

impl MessageBrokerMessageKind {
    pub fn routing_key(&self) -> String {
        match self {
            MessageBrokerMessageKind::Event => ApplicationQueueName::Events.to_string(),
            MessageBrokerMessageKind::Job(priority) => match priority {
                MessageBrokerMessageKindJobPriority::Critical => {
                    ApplicationQueueName::JobsCritical.to_string()
                }
                MessageBrokerMessageKindJobPriority::Normal => {
                    ApplicationQueueName::JobsNormal.to_string()
                }
                MessageBrokerMessageKindJobPriority::Background => {
                    ApplicationQueueName::JobsBackground.to_string()
                }
            },
        }
    }
}

pub trait MessageBrokerMessage: Serialize + Send + Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> MessageBrokerMessageKind;
}

#[async_trait]
pub trait MessageBrokerPublisher: Send + Sync {
    async fn publish(
        &self,
        message: &dyn MessageBrokerMessage,
    ) -> Result<(), MessageBrokerPublisherPublishError>;
}

erased_serde::serialize_trait_object!(MessageBrokerMessage);
