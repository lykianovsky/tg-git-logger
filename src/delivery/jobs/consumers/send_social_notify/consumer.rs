use crate::application::notification::commands::send_social_notify::command::SendSocialNotifyExecutorCommand;
use crate::application::notification::commands::send_social_notify::executor::SendSocialNotifyExecutor;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::shared::command::CommandExecutor;
use crate::infrastructure::processing::job::{JobConsumer, JobConsumerError, JobConsumerResponse};
use async_trait::async_trait;
use std::sync::Arc;

pub struct SendSocialNotifyJobConsumer {
    pub executor: Arc<SendSocialNotifyExecutor>,
}

#[async_trait]
impl JobConsumer for SendSocialNotifyJobConsumer {
    fn name(&self) -> &'static str {
        SendSocialNotifyJob::NAME
    }

    async fn run(&self, payload: &[u8]) -> Result<JobConsumerResponse, JobConsumerError> {
        let payload: SendSocialNotifyJob = serde_json::from_slice(payload)
            .map_err(|e| JobConsumerError::DeserializationError(e.to_string()))?;

        tracing::debug!(chat_id = %payload.chat_id.0, social_type = ?payload.social_type, "Processing send_social_notify job");

        if let Err(e) = self
            .executor
            .execute(&SendSocialNotifyExecutorCommand {
                message: payload.message,
                chat_id: payload.chat_id,
                social_type: payload.social_type,
            })
            .await
        {
            tracing::error!(error = %e, "send_social_notify failed, scheduling retry");
            return Ok(JobConsumerResponse::Retry(e.to_string()));
        };

        Ok(JobConsumerResponse::Ok)
    }
}
