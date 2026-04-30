use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;

pub struct CompleteReleasePlanExecutorCommand {
    pub plan_id: ReleasePlanId,
}
