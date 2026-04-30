use crate::application::release_plan::commands::complete_release_plan::command::CompleteReleasePlanExecutorCommand;
use crate::application::release_plan::commands::complete_release_plan::error::CompleteReleasePlanExecutorError;
use crate::application::release_plan::commands::complete_release_plan::response::CompleteReleasePlanExecutorResponse;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::release_plan::value_objects::release_plan_status::ReleasePlanStatus;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct CompleteReleasePlanExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
}

impl CommandExecutor for CompleteReleasePlanExecutor {
    type Command = CompleteReleasePlanExecutorCommand;
    type Response = CompleteReleasePlanExecutorResponse;
    type Error = CompleteReleasePlanExecutorError;

    #[tracing::instrument(
        name = "complete_release_plan",
        skip_all,
        fields(plan_id = cmd.plan_id.0)
    )]
    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.release_plan_repo
            .set_status(cmd.plan_id, ReleasePlanStatus::Done)
            .await?;
        Ok(CompleteReleasePlanExecutorResponse)
    }
}
