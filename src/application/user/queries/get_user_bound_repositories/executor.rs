use crate::application::user::queries::get_user_bound_repositories::error::GetUserBoundRepositoriesError;
use crate::application::user::queries::get_user_bound_repositories::query::GetUserBoundRepositoriesQuery;
use crate::application::user::queries::get_user_bound_repositories::response::GetUserBoundRepositoriesResponse;
use crate::domain::repository::repositories::repository_repository::{
    FindRepositoryByIdError, RepositoryRepository,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_connection_repositories_repository::UserConnectionRepositoriesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::{
    FindSocialServiceByIdError, UserSocialAccountsRepository,
};
use std::sync::Arc;

pub struct GetUserBoundRepositoriesExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
}

impl GetUserBoundRepositoriesExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_connection_repositories_repo,
            repository_repo,
        }
    }
}

impl CommandExecutor for GetUserBoundRepositoriesExecutor {
    type Command = GetUserBoundRepositoriesQuery;
    type Response = GetUserBoundRepositoriesResponse;
    type Error = GetUserBoundRepositoriesError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await
            .map_err(|e| match e {
                FindSocialServiceByIdError::NotFound => GetUserBoundRepositoriesError::UserNotFound,
                FindSocialServiceByIdError::DbError(msg) => {
                    GetUserBoundRepositoriesError::DbError(msg)
                }
            })?;

        let connections = self
            .user_connection_repositories_repo
            .find_by_user_id(social.user_id)
            .await
            .map_err(|e| GetUserBoundRepositoriesError::DbError(e.to_string()))?;

        let mut repositories = Vec::with_capacity(connections.len());
        for conn in connections {
            let repo = self
                .repository_repo
                .find_by_id(conn.repository_id)
                .await
                .map_err(|e| match e {
                    FindRepositoryByIdError::NotFound => GetUserBoundRepositoriesError::DbError(
                        format!("Repository {} not found", conn.repository_id.0),
                    ),
                    FindRepositoryByIdError::DbError(msg) => {
                        GetUserBoundRepositoriesError::DbError(msg)
                    }
                })?;
            repositories.push(repo);
        }

        Ok(GetUserBoundRepositoriesResponse { repositories })
    }
}
