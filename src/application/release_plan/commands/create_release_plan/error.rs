use crate::domain::release_plan::repositories::release_plan_repository::CreateReleasePlanError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateReleasePlanExecutorError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for CreateReleasePlanExecutorError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::UserNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<CreateReleasePlanError> for CreateReleasePlanExecutorError {
    fn from(e: CreateReleasePlanError) -> Self {
        match e {
            CreateReleasePlanError::DbError(msg) => Self::DbError(msg),
        }
    }
}
