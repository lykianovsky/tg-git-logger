use crate::application::user::queries::get_user_overview::error::GetUserOverviewError;
use crate::application::user::queries::get_user_overview::query::GetUserOverviewQuery;
use crate::application::user::queries::get_user_overview::response::GetUserOverviewResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_connection_repositories_repository::UserConnectionRepositoriesRepository;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use std::sync::Arc;

pub struct GetUserOverviewExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
}

impl GetUserOverviewExecutor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
        user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
        user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_has_roles_repo,
            user_vc_accounts_repo,
            user_preferences_repo,
            user_connection_repositories_repo,
            repository_repo,
        }
    }
}

impl CommandExecutor for GetUserOverviewExecutor {
    type Command = GetUserOverviewQuery;
    type Response = GetUserOverviewResponse;
    type Error = GetUserOverviewError;

    #[tracing::instrument(
        name = "get_user_overview",
        skip_all,
        fields(social_user_id = cmd.social_user_id.0)
    )]
    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;
        let user_id = social.user_id;

        let github_login = self
            .user_vc_accounts_repo
            .find_by_user_id(&user_id)
            .await
            .ok()
            .map(|a| a.version_control_login);

        let roles = self
            .user_has_roles_repo
            .get_all(user_id)
            .await
            .map(|rs| rs.into_iter().map(|r| r.name).collect())
            .unwrap_or_default();

        let prefs = self
            .user_preferences_repo
            .find_by_user_id(user_id)
            .await
            .ok()
            .flatten();

        let (dnd_window, timezone, vacation_until, snooze_until) = match prefs {
            Some(p) => (p.dnd_window, p.timezone, p.vacation_until, p.snooze_until),
            None => (None, None, None, None),
        };

        let connections = self
            .user_connection_repositories_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| GetUserOverviewError::DbError(e.to_string()))?;

        let mut repositories = Vec::with_capacity(connections.len());
        for conn in connections {
            if let Ok(repo) = self.repository_repo.find_by_id(conn.repository_id).await {
                repositories.push(repo);
            }
        }

        Ok(GetUserOverviewResponse {
            github_login,
            roles,
            dnd_window,
            timezone,
            vacation_until,
            snooze_until,
            repositories,
        })
    }
}
