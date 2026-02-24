use crate::delivery::jobs::consumers::send_email::payload::SendEmailJob;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerMessage;
use crate::infrastructure::processing::job::{JobConsumer, JobConsumerError, JobConsumerResponse};
use async_trait::async_trait;

pub struct SendEmailJobConsumer {}

#[async_trait]
impl JobConsumer for SendEmailJobConsumer {
    fn name(&self) -> &'static str {
        SendEmailJob::NAME
    }

    async fn run(&self, payload: &[u8]) -> Result<JobConsumerResponse, JobConsumerError> {
        let payload: SendEmailJob = serde_json::from_slice(payload)
            .map_err(|e| JobConsumerError::DeserializationError(e.to_string()))?;

        tracing::info!("Sending email job payload: {:?}", payload);

        Ok(JobConsumerResponse::Ok)
    }
}
