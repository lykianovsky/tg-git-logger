use crate::application::repository::commands::create_repository::command::CreateRepositoryCommand;
use crate::application::repository::commands::create_repository::error::CreateRepositoryExecutorError;
use crate::application::repository::commands::create_repository::response::CreateRepositoryResponse;
use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::repositories::repository_repository::{
    CreateRepositoryError, RepositoryRepository,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::DatabaseConnection;
use sea_orm::TransactionTrait;
use std::sync::Arc;

pub struct CreateRepositoryExecutor {
    pub db: Arc<DatabaseConnection>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
}

impl CreateRepositoryExecutor {
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

impl CommandExecutor for CreateRepositoryExecutor {
    type Command = CreateRepositoryCommand;
    type Response = CreateRepositoryResponse;
    type Error = CreateRepositoryExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| CreateRepositoryExecutorError::DbError(e.to_string()))?;

        let placeholder = Repository {
            id: RepositoryId(0),
            name: cmd.name.clone(),
            owner: cmd.owner.clone(),
            url: cmd.url.clone(),
            social_chat_id: None,
            notifications_chat_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let repository = self
            .repository_repo
            .create(&txn, &placeholder)
            .await
            .map_err(|e| match e {
                CreateRepositoryError::DbError(msg) => CreateRepositoryExecutorError::DbError(msg),
            })?;

        txn.commit()
            .await
            .map_err(|e| CreateRepositoryExecutorError::DbError(e.to_string()))?;

        Ok(CreateRepositoryResponse { repository })
    }
}
