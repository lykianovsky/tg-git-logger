use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueue;
use crate::infrastructure::processing::job::{JobConsumerError, JobConsumerResponse};
use crate::infrastructure::processing::worker::{
    MessageBrokerWorker, MessageBrokerWorkerStartError,
};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;

pub struct MessageBrokerJobsWorker {
    name: String,
    queue: Arc<MessageBrokerQueue>,
    jobs_registry: Arc<JobConsumersRegistry>,
    message_broker: Arc<dyn MessageBroker>,
}

impl MessageBrokerJobsWorker {
    pub fn new(
        name: &str,
        queue: Arc<MessageBrokerQueue>,
        jobs_registry: Arc<JobConsumersRegistry>,
        message_broker: Arc<dyn MessageBroker>,
    ) -> Self {
        Self {
            name: name.to_string(),
            queue,
            jobs_registry,
            message_broker,
        }
    }
}

#[async_trait]
impl MessageBrokerWorker for MessageBrokerJobsWorker {
    async fn start(self) -> Result<(), MessageBrokerWorkerStartError> {
        let queue = &*self.queue.clone();

        let mut stream = self
            .message_broker
            .consume(self.name.as_str(), queue)
            .await
            .map_err(|_e| MessageBrokerWorkerStartError::Test)?;

        while let Some(delivery) = stream.next().await {
            tracing::debug!(
                "MessageBrokerCommandWorker, FROM_WORKER {}: {}",
                self.name,
                delivery.envelope.name
            );

            let handler = match self.jobs_registry.get(&delivery.envelope.name).await {
                Some(v) => v,
                None => {
                    let error = format!(
                        "Failed get registry job from name: {}",
                        &delivery.envelope.name
                    );
                    tracing::error!(error);
                    delivery.reject(error.as_str()).await;
                    continue;
                }
            };

            match handler.run(&delivery.envelope.payload).await {
                Ok(response) => match response {
                    JobConsumerResponse::Ok => delivery.ack().await,
                    JobConsumerResponse::Reject(reason) => delivery.reject(reason.as_str()).await,
                    JobConsumerResponse::Requeue => delivery.requeue().await,
                    JobConsumerResponse::Retry(reason) => delivery.retry(reason.as_str()).await,
                },
                Err(error) => {
                    tracing::error!(error = %error, "Failed handle run from job");

                    match error {
                        JobConsumerError::DeserializationError(e) => {
                            delivery.reject(e.as_str()).await
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
