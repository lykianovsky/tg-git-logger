use crate::application::user_preferences::queries::get_user_preferences::error::GetUserPreferencesError;
use crate::application::user_preferences::queries::get_user_preferences::query::GetUserPreferencesQuery;
use crate::application::user_preferences::queries::get_user_preferences::response::GetUserPreferencesResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use std::sync::Arc;

pub struct GetUserPreferencesExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
}

impl GetUserPreferencesExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_preferences_repo,
        }
    }
}

impl CommandExecutor for GetUserPreferencesExecutor {
    type Command = GetUserPreferencesQuery;
    type Response = GetUserPreferencesResponse;
    type Error = GetUserPreferencesError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_account = self
            .user_socials_repo
            .find_by_social_user_id(&query.social_user_id)
            .await?;

        let preferences = self
            .user_preferences_repo
            .find_by_user_id(social_account.user_id)
            .await?;

        Ok(GetUserPreferencesResponse { preferences })
    }
}
