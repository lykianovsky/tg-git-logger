use crate::application::task::queries::list_task_tracker_options::error::ListTaskTrackerOptionsError;
use crate::application::task::queries::list_task_tracker_options::query::ListTaskTrackerOptionsQuery;
use crate::application::task::queries::list_task_tracker_options::response::{
    ListTaskTrackerOptionsResponse, TaskTrackerOption,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use std::sync::Arc;

pub struct ListTaskTrackerOptionsExecutor {
    task_tracker_client: Arc<dyn TaskTrackerClient>,
}

impl ListTaskTrackerOptionsExecutor {
    pub fn new(task_tracker_client: Arc<dyn TaskTrackerClient>) -> Self {
        Self {
            task_tracker_client,
        }
    }
}

impl CommandExecutor for ListTaskTrackerOptionsExecutor {
    type Command = ListTaskTrackerOptionsQuery;
    type Response = ListTaskTrackerOptionsResponse;
    type Error = ListTaskTrackerOptionsError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let options = match query {
            ListTaskTrackerOptionsQuery::Users => self
                .task_tracker_client
                .list_users()
                .await?
                .into_iter()
                .map(|user| TaskTrackerOption {
                    id: user.id,
                    name: user.name,
                })
                .collect(),
            ListTaskTrackerOptionsQuery::Tags => self
                .task_tracker_client
                .list_tags()
                .await?
                .into_iter()
                .map(|tag| TaskTrackerOption {
                    id: tag.id,
                    name: tag.name,
                })
                .collect(),
        };

        Ok(ListTaskTrackerOptionsResponse { options })
    }
}
