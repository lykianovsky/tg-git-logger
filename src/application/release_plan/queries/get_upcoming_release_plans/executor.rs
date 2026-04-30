use crate::application::release_plan::queries::get_upcoming_release_plans::error::GetUpcomingReleasePlansError;
use crate::application::release_plan::queries::get_upcoming_release_plans::query::GetUpcomingReleasePlansQuery;
use crate::application::release_plan::queries::get_upcoming_release_plans::response::GetUpcomingReleasePlansResponse;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct GetUpcomingReleasePlansExecutor {
    release_plan_repo: Arc<dyn ReleasePlanRepository>,
}

impl GetUpcomingReleasePlansExecutor {
    pub fn new(release_plan_repo: Arc<dyn ReleasePlanRepository>) -> Self {
        Self { release_plan_repo }
    }
}

impl CommandExecutor for GetUpcomingReleasePlansExecutor {
    type Command = GetUpcomingReleasePlansQuery;
    type Response = GetUpcomingReleasePlansResponse;
    type Error = GetUpcomingReleasePlansError;

    #[tracing::instrument(
        name = "get_upcoming_release_plans",
        skip_all,
        fields(from_date = %cmd.from_date)
    )]
    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let plans = self.release_plan_repo.find_upcoming(cmd.from_date).await?;
        Ok(GetUpcomingReleasePlansResponse { plans })
    }
}
