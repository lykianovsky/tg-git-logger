use crate::application::task::queries::get_task_card::error::GetTaskCardError;
use crate::application::task::queries::get_task_card::query::GetTaskCardQuery;
use crate::application::task::queries::get_task_card::response::GetTaskCardResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::ports::task_tracker_client::{
    TaskTrackerClient, TaskTrackerClientGetCardError,
};
use std::sync::Arc;

pub struct GetTaskCardExecutor {
    pub task_tracker_client: Arc<dyn TaskTrackerClient>,
}

impl GetTaskCardExecutor {
    pub fn new(task_tracker_client: Arc<dyn TaskTrackerClient>) -> Self {
        Self {
            task_tracker_client,
        }
    }
}

impl CommandExecutor for GetTaskCardExecutor {
    type Command = GetTaskCardQuery;
    type Response = GetTaskCardResponse;
    type Error = GetTaskCardError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let card = self
            .task_tracker_client
            .get_card(cmd.task_id)
            .await
            .map_err(|e| match e {
                TaskTrackerClientGetCardError::NotFound => GetTaskCardError::NotFound,
                TaskTrackerClientGetCardError::ClientError(msg) => {
                    GetTaskCardError::ClientError(msg)
                }
            })?;

        Ok(GetTaskCardResponse {
            title: card.title,
            url: card.url,
        })
    }
}
