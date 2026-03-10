use crate::application::task::commands::move_task_to_test::command::MoveTaskToTestExecutorCommand;
use crate::application::task::commands::move_task_to_test::executor::MoveTaskToTestExecutor;
use crate::delivery::jobs::consumers::move_task_to_test::payload::MoveTaskToTestJob;
use crate::domain::shared::command::CommandExecutor;
use crate::infrastructure::processing::job::{JobConsumer, JobConsumerError, JobConsumerResponse};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MoveTaskToTestJobConsumer {
    pub executor: Arc<MoveTaskToTestExecutor>,
}

#[async_trait]
impl JobConsumer for MoveTaskToTestJobConsumer {
    fn name(&self) -> &'static str {
        MoveTaskToTestJob::NAME
    }

    async fn run(&self, payload: &[u8]) -> Result<JobConsumerResponse, JobConsumerError> {
        let payload: MoveTaskToTestJob = serde_json::from_slice(payload)
            .map_err(|e| JobConsumerError::DeserializationError(e.to_string()))?;

        tracing::info!("Sending task to column job payload: {:?}", payload);

        if let Err(e) = self
            .executor
            .execute(&MoveTaskToTestExecutorCommand {
                task_id: payload.task_id,
            })
            .await
        {
            return Ok(JobConsumerResponse::Retry(e.to_string()));
        };

        Ok(JobConsumerResponse::Ok)
    }
}
