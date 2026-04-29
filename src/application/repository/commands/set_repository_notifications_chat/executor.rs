use crate::application::repository::commands::set_repository_notifications_chat::command::SetRepositoryNotificationsChatCommand;
use crate::application::repository::commands::set_repository_notifications_chat::error::SetRepositoryNotificationsChatError;
use crate::application::repository::commands::set_repository_notifications_chat::response::SetRepositoryNotificationsChatResponse;
use crate::domain::repository::repositories::repository_repository::{
    FindRepositoryByIdError, RepositoryRepository, UpdateRepositoryError,
};
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct SetRepositoryNotificationsChatExecutor {
    pub db: Arc<DatabaseConnection>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
}

impl SetRepositoryNotificationsChatExecutor {
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

impl CommandExecutor for SetRepositoryNotificationsChatExecutor {
    type Command = SetRepositoryNotificationsChatCommand;
    type Response = SetRepositoryNotificationsChatResponse;
    type Error = SetRepositoryNotificationsChatError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut repository = self
            .repository_repo
            .find_by_id(cmd.repository_id)
            .await
            .map_err(|e| match e {
                FindRepositoryByIdError::NotFound => SetRepositoryNotificationsChatError::NotFound,
                FindRepositoryByIdError::DbError(msg) => {
                    SetRepositoryNotificationsChatError::DbError(msg)
                }
            })?;

        repository.notifications_chat_id = Some(cmd.notifications_chat_id);
        repository.updated_at = Utc::now();

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| SetRepositoryNotificationsChatError::DbError(e.to_string()))?;

        let repository = self
            .repository_repo
            .update(&txn, &repository)
            .await
            .map_err(|e| match e {
                UpdateRepositoryError::NotFound => SetRepositoryNotificationsChatError::NotFound,
                UpdateRepositoryError::DbError(msg) => {
                    SetRepositoryNotificationsChatError::DbError(msg)
                }
            })?;

        txn.commit()
            .await
            .map_err(|e| SetRepositoryNotificationsChatError::DbError(e.to_string()))?;

        Ok(SetRepositoryNotificationsChatResponse { repository })
    }
}
