use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueue;
use crate::infrastructure::processing::event_bus::EventBus;
use crate::infrastructure::processing::worker::{
    MessageBrokerWorker, MessageBrokerWorkerStartError,
};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

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

    async fn start(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Result<(), MessageBrokerWorkerStartError> {
        let queue = &*self.queue.clone();

        let mut stream = self
            .message_broker
            .consume(self.name.as_str(), queue)
            .await
            .map_err(|e| {
                MessageBrokerWorkerStartError::FailedToCreateConsumerStream(e.to_string())
            })?;

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    tracing::info!(worker = %self.name, "Worker cancelled, shutting down");
                    break;
                }
                delivery = stream.next() => {
                    let delivery = match delivery {
                        Some(d) => d,
                        None => {
                            tracing::warn!(worker = %self.name, "Stream closed");
                            break;
                        }
                    };

                    tracing::debug!(
                        worker = %self.name,
                        event = %delivery.envelope.name,
                        "Received event"
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
            }
        }

        Ok(())
    }
}
