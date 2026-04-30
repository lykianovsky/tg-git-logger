use crate::application::release_plan::commands::update_release_plan::command::{
    ReleasePlanPatch, UpdateReleasePlanExecutorCommand,
};
use crate::application::release_plan::commands::update_release_plan::error::UpdateReleasePlanExecutorError;
use crate::application::release_plan::commands::update_release_plan::response::UpdateReleasePlanExecutorResponse;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct UpdateReleasePlanExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
}

impl CommandExecutor for UpdateReleasePlanExecutor {
    type Command = UpdateReleasePlanExecutorCommand;
    type Response = UpdateReleasePlanExecutorResponse;
    type Error = UpdateReleasePlanExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        if let ReleasePlanPatch::SetRepositories { ids } = &cmd.patch {
            self.release_plan_repo
                .set_repositories(cmd.plan_id, ids.clone())
                .await?;
            let plan = self.release_plan_repo.find_by_id(cmd.plan_id).await?;
            return Ok(UpdateReleasePlanExecutorResponse { plan });
        }

        let mut plan = self.release_plan_repo.find_by_id(cmd.plan_id).await?;

        match &cmd.patch {
            ReleasePlanPatch::SetPlannedDate { date } => plan.planned_date = *date,
            ReleasePlanPatch::SetCallDateTime { datetime } => plan.call_datetime = Some(*datetime),
            ReleasePlanPatch::ClearCallDateTime => {
                plan.call_datetime = None;
                plan.meeting_url = None;
                plan.notified_call_at = None;
            }
            ReleasePlanPatch::SetMeetingUrl { url } => plan.meeting_url = Some(url.clone()),
            ReleasePlanPatch::ClearMeetingUrl => plan.meeting_url = None,
            ReleasePlanPatch::SetNote { text } => plan.note = Some(text.clone()),
            ReleasePlanPatch::ClearNote => plan.note = None,
            ReleasePlanPatch::SetRepositories { .. } => unreachable!(),
        }

        let updated = self.release_plan_repo.update_fields(&plan).await?;
        Ok(UpdateReleasePlanExecutorResponse { plan: updated })
    }
}
