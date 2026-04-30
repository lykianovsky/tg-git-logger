use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct CancelReleasePlanExecutorCommand {
    pub plan_id: ReleasePlanId,
    pub cancelled_by_social_user_id: SocialUserId,
    pub reason: String,
}
