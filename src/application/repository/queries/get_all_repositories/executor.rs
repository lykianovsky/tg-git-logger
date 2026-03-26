use crate::application::repository::queries::get_all_repositories::error::GetAllRepositoriesError;
use crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery;
use crate::application::repository::queries::get_all_repositories::response::GetAllRepositoriesResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct GetAllRepositoriesExecutor {
    repository_repo: Arc<dyn RepositoryRepository>,
}

impl GetAllRepositoriesExecutor {
    pub fn new(repository_repo: Arc<dyn RepositoryRepository>) -> Self {
        Self { repository_repo }
    }
}

impl CommandExecutor for GetAllRepositoriesExecutor {
    type Command = GetAllRepositoriesQuery;
    type Response = GetAllRepositoriesResponse;
    type Error = GetAllRepositoriesError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let repositories = self
            .repository_repo
            .find_all()
            .await
            .map_err(|e| GetAllRepositoriesError::DbError(e.to_string()))?;

        Ok(GetAllRepositoriesResponse { repositories })
    }
}
