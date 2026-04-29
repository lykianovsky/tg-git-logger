use crate::domain::release_plan::entities::release_plan::{
    NewReleasePlan, ReleasePlan, ReleasePlanNotificationKind,
};
use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
use crate::domain::release_plan::value_objects::release_plan_status::ReleasePlanStatus;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateReleasePlanError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindReleasePlanError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Release plan not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateReleasePlanError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait ReleasePlanRepository: Send + Sync {
    async fn create(&self, plan: &NewReleasePlan) -> Result<ReleasePlan, CreateReleasePlanError>;

    async fn find_by_id(&self, id: ReleasePlanId) -> Result<ReleasePlan, FindReleasePlanError>;

    async fn find_active(&self) -> Result<Vec<ReleasePlan>, FindReleasePlanError>;

    async fn set_status(
        &self,
        id: ReleasePlanId,
        status: ReleasePlanStatus,
    ) -> Result<(), UpdateReleasePlanError>;

    async fn mark_notified(
        &self,
        id: ReleasePlanId,
        kind: ReleasePlanNotificationKind,
        at: DateTime<Utc>,
    ) -> Result<(), UpdateReleasePlanError>;
}
