use crate::domain::release_plan::entities::release_plan::ReleasePlan;

pub struct GetUpcomingReleasePlansResponse {
    pub plans: Vec<ReleasePlan>,
}
