use crate::application::repository::commands::update_repository::command::UpdateRepositoryCommand;
use crate::application::repository::commands::update_repository::error::UpdateRepositoryExecutorError;
use crate::application::repository::commands::update_repository::response::UpdateRepositoryResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct UpdateRepositoryExecutor {
    db: Arc<DatabaseConnection>,
    repository_repo: Arc<dyn RepositoryRepository>,
}

impl UpdateRepositoryExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        repository_repo: Arc<dyn RepositoryRepository>,
    ) -> Self {
        Self {
            db,
            repository_repo,
        }
    }
}

impl CommandExecutor for UpdateRepositoryExecutor {
    type Command = UpdateRepositoryCommand;
    type Response = UpdateRepositoryResponse;
    type Error = UpdateRepositoryExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut repository = self.repository_repo.find_by_id(cmd.id).await?;

        repository.name = cmd.name.clone();
        repository.owner = cmd.owner.clone();
        repository.url = cmd.url.clone();
        repository.updated_at = Utc::now();

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| UpdateRepositoryExecutorError::DbError(e.to_string()))?;

        let repository = self.repository_repo.update(&txn, &repository).await?;

        txn.commit()
            .await
            .map_err(|e| UpdateRepositoryExecutorError::DbError(e.to_string()))?;

        Ok(UpdateRepositoryResponse { repository })
    }
}
