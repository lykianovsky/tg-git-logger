use crate::domain::release_plan::repositories::release_plan_repository::{
    FindReleasePlanError, UpdateReleasePlanError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateReleasePlanExecutorError {
    #[error("Release plan not found")]
    NotFound,
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindReleasePlanError> for UpdateReleasePlanExecutorError {
    fn from(e: FindReleasePlanError) -> Self {
        match e {
            FindReleasePlanError::NotFound => Self::NotFound,
            FindReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpdateReleasePlanError> for UpdateReleasePlanExecutorError {
    fn from(e: UpdateReleasePlanError) -> Self {
        match e {
            UpdateReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}
