use crate::domain::user::repositories::user_repository::SetUserActiveError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeactivateUserExecutorError {
    #[error("User social account not found")]
    SocialAccountNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for DeactivateUserExecutorError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::SocialAccountNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<SetUserActiveError> for DeactivateUserExecutorError {
    fn from(e: SetUserActiveError) -> Self {
        match e {
            SetUserActiveError::NotFound => Self::UserNotFound,
            SetUserActiveError::DbError(msg) => Self::DbError(msg),
        }
    }
}
