use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use chrono::{DateTime, NaiveDate, Utc};

pub enum ReleasePlanPatch {
    SetPlannedDate { date: NaiveDate },
    SetCallDateTime { datetime: DateTime<Utc> },
    ClearCallDateTime,
    SetMeetingUrl { url: String },
    ClearMeetingUrl,
    SetNote { text: String },
    ClearNote,
    SetRepositories { ids: Vec<RepositoryId> },
}

pub struct UpdateReleasePlanExecutorCommand {
    pub plan_id: ReleasePlanId,
    pub patch: ReleasePlanPatch,
}
