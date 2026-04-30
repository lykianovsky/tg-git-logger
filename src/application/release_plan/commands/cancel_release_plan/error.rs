use crate::domain::release_plan::repositories::release_plan_repository::{
    FindReleasePlanError, UpdateReleasePlanError,
};
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CancelReleasePlanExecutorError {
    #[error("Release plan not found")]
    NotFound,
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindReleasePlanError> for CancelReleasePlanExecutorError {
    fn from(e: FindReleasePlanError) -> Self {
        match e {
            FindReleasePlanError::NotFound => Self::NotFound,
            FindReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpdateReleasePlanError> for CancelReleasePlanExecutorError {
    fn from(e: UpdateReleasePlanError) -> Self {
        match e {
            UpdateReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<FindSocialServiceByIdError> for CancelReleasePlanExecutorError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::DbError("Social account not found".to_string()),
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}
