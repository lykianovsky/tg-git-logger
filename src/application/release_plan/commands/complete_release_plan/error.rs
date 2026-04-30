use crate::domain::release_plan::repositories::release_plan_repository::UpdateReleasePlanError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompleteReleasePlanExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<UpdateReleasePlanError> for CompleteReleasePlanExecutorError {
    fn from(e: UpdateReleasePlanError) -> Self {
        match e {
            UpdateReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}
