use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user_preferences::entities::user_preferences::UserPreferences;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FindUserPreferencesError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum UpsertUserPreferencesError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait UserPreferencesRepository: Send + Sync {
    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserPreferences>, FindUserPreferencesError>;

    async fn upsert(
        &self,
        prefs: &UserPreferences,
    ) -> Result<UserPreferences, UpsertUserPreferencesError>;
}
