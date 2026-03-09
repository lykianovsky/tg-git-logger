use async_trait::async_trait;
use erased_serde::Serialize;

pub enum MessageBrokerPublisherPublishError {
    SerializationFailed(String),
    PublishCreateFailed(String),
    PublishConfirmFailed(String),
}

pub enum MessageBrokerMessageKind {
    Event,
    Job,
}

impl MessageBrokerMessageKind {
    pub fn routing_key(&self) -> &str {
        match self {
            MessageBrokerMessageKind::Event => "events",
            MessageBrokerMessageKind::Job => "jobs",
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
