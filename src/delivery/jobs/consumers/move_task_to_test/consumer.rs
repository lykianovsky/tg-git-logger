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

    #[tracing::instrument(name = "job.move_task_to_test", skip(self, payload))]
    async fn run(&self, payload: &[u8]) -> Result<JobConsumerResponse, JobConsumerError> {
        let payload: MoveTaskToTestJob = serde_json::from_slice(payload)
            .map_err(|e| JobConsumerError::DeserializationError(e.to_string()))?;

        tracing::debug!(task_id = %payload.task_id.0, "Processing move_task_to_test job");

        if let Err(e) = self
            .executor
            .execute(&MoveTaskToTestExecutorCommand {
                task_id: payload.task_id,
                column_id: payload.column_id,
            })
            .await
        {
            tracing::error!(task_id = %payload.task_id.0, error = %e, "move_task_to_test failed, scheduling retry");
            return Ok(JobConsumerResponse::Retry(e.to_string()));
        };

        Ok(JobConsumerResponse::Ok)
    }
}
