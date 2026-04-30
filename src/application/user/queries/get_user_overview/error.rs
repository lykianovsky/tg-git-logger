use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetUserOverviewError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for GetUserOverviewError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::UserNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}
