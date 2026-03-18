use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueue;

use crate::infrastructure::processing::event_bus::EventBus;
use crate::infrastructure::processing::worker::{
    MessageBrokerWorker, MessageBrokerWorkerStartError,
};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;

pub struct MessageBrokerEventsWorker {
    name: String,
    queue: Arc<MessageBrokerQueue>,
    event_bus: Arc<EventBus>,
    message_broker: Arc<dyn MessageBroker>,
}

impl MessageBrokerEventsWorker {
    pub fn new(
        name: &str,
        queue: Arc<MessageBrokerQueue>,
        event_bus: Arc<EventBus>,
        message_broker: Arc<dyn MessageBroker>,
    ) -> Self {
        Self {
            name: name.to_string(),
            queue,
            event_bus,
            message_broker,
        }
    }
}

#[async_trait]
impl MessageBrokerWorker for MessageBrokerEventsWorker {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    async fn start(self: Box<Self>) -> Result<(), MessageBrokerWorkerStartError> {
        let queue = &*self.queue.clone();

        let mut stream = self
            .message_broker
            .consume(self.name.as_str(), queue)
            .await
            .map_err(|e| {
                MessageBrokerWorkerStartError::FailedToCreateConsumerStream(e.to_string())
            })?;

        while let Some(delivery) = stream.next().await {
            tracing::debug!(
                "MessageBrokerEventsWorker, FROM_WORKER {}: {}",
                self.name,
                delivery.envelope.name
            );

            if let Err(e) = self
                .event_bus
                .dispatch_raw(&delivery.envelope.name, &delivery.envelope.payload)
                .await
            {
                tracing::error!(
                    event = %delivery.envelope.name,
                    error = %e,
                    "Failed to dispatch event"
                );

                delivery.reject(&e.to_string()).await;
                continue;
            }

            delivery.ack().await;
        }

        Ok(())
    }
}
