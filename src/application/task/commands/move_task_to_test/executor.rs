use crate::application::task::commands::move_task_to_test::command::MoveTaskToTestExecutorCommand;
use crate::application::task::commands::move_task_to_test::error::MoveTaskToTestExecutorError;
use crate::application::task::commands::move_task_to_test::response::MoveTaskToTestExecutorResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use std::sync::Arc;

pub struct MoveTaskToTestExecutor {
    task_tracker_client: Arc<dyn TaskTrackerClient>,
    task_tracker_service: Arc<dyn TaskTrackerService>,
}

impl MoveTaskToTestExecutor {
    pub fn new(
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        task_tracker_service: Arc<dyn TaskTrackerService>,
    ) -> Self {
        Self {
            task_tracker_client,
            task_tracker_service,
        }
    }
}

impl CommandExecutor for MoveTaskToTestExecutor {
    type Command = MoveTaskToTestExecutorCommand;
    type Response = MoveTaskToTestExecutorResponse;
    type Error = MoveTaskToTestExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.task_tracker_client
            .move_task_to_column(cmd.task_id, cmd.column_id)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    task_id = %cmd.task_id.0,
                    column_id = %cmd.column_id,
                    error = %e,
                    "Failed to move task to test column"
                );
            })?;

        tracing::info!(task_id = %cmd.task_id.0, column_id = %cmd.column_id, "Task moved to test column");

        Ok(MoveTaskToTestExecutorResponse {})
    }
}
