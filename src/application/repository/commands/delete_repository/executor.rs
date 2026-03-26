use crate::application::repository::commands::delete_repository::command::DeleteRepositoryCommand;
use crate::application::repository::commands::delete_repository::error::DeleteRepositoryExecutorError;
use crate::application::repository::commands::delete_repository::response::DeleteRepositoryResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct DeleteRepositoryExecutor {
    db: Arc<DatabaseConnection>,
    repository_repo: Arc<dyn RepositoryRepository>,
}

impl DeleteRepositoryExecutor {
    pub fn new(db: Arc<DatabaseConnection>, repository_repo: Arc<dyn RepositoryRepository>) -> Self {
        Self { db, repository_repo }
    }
}

impl CommandExecutor for DeleteRepositoryExecutor {
    type Command = DeleteRepositoryCommand;
    type Response = DeleteRepositoryResponse;
    type Error = DeleteRepositoryExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| DeleteRepositoryExecutorError::DbError(e.to_string()))?;

        self.repository_repo.delete(&txn, cmd.id).await?;

        txn.commit()
            .await
            .map_err(|e| DeleteRepositoryExecutorError::DbError(e.to_string()))?;

        Ok(DeleteRepositoryResponse)
    }
}
