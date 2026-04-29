use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use crate::domain::user_preferences::repositories::user_preferences_repository::{
    FindUserPreferencesError, UpsertUserPreferencesError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateUserPreferencesExecutorError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DbError(String),
}

impl From<FindSocialServiceByIdError> for UpdateUserPreferencesExecutorError {
    fn from(e: FindSocialServiceByIdError) -> Self {
        match e {
            FindSocialServiceByIdError::NotFound => Self::UserNotFound,
            FindSocialServiceByIdError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<FindUserPreferencesError> for UpdateUserPreferencesExecutorError {
    fn from(e: FindUserPreferencesError) -> Self {
        match e {
            FindUserPreferencesError::DbError(msg) => Self::DbError(msg),
        }
    }
}

impl From<UpsertUserPreferencesError> for UpdateUserPreferencesExecutorError {
    fn from(e: UpsertUserPreferencesError) -> Self {
        match e {
            UpsertUserPreferencesError::DbError(msg) => Self::DbError(msg),
        }
    }
}
