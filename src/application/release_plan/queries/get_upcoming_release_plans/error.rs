use crate::domain::release_plan::repositories::release_plan_repository::FindReleasePlanError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUpcomingReleasePlansError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindReleasePlanError> for GetUpcomingReleasePlansError {
    fn from(e: FindReleasePlanError) -> Self {
        match e {
            FindReleasePlanError::DbError(msg) => Self::DbError(msg),
            FindReleasePlanError::NotFound => Self::DbError("Not found".to_string()),
        }
    }
}
