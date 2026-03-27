use crate::application::repository::commands::unset_repository_notification_chat::command::UnsetRepositoryNotificationChatCommand;
use crate::application::repository::commands::unset_repository_notification_chat::error::UnsetRepositoryNotificationChatError;
use crate::application::repository::commands::unset_repository_notification_chat::response::UnsetRepositoryNotificationChatResponse;
use crate::domain::repository::repositories::repository_repository::{
    FindRepositoryByIdError, RepositoryRepository, UpdateRepositoryError,
};
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct UnsetRepositoryNotificationChatExecutor {
    pub db: Arc<DatabaseConnection>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
}

impl UnsetRepositoryNotificationChatExecutor {
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

impl CommandExecutor for UnsetRepositoryNotificationChatExecutor {
    type Command = UnsetRepositoryNotificationChatCommand;
    type Response = UnsetRepositoryNotificationChatResponse;
    type Error = UnsetRepositoryNotificationChatError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut repository = self
            .repository_repo
            .find_by_id(cmd.repository_id)
            .await
            .map_err(|e| match e {
                FindRepositoryByIdError::NotFound => UnsetRepositoryNotificationChatError::NotFound,
                FindRepositoryByIdError::DbError(msg) => {
                    UnsetRepositoryNotificationChatError::DbError(msg)
                }
            })?;

        repository.social_chat_id = None;
        repository.updated_at = Utc::now();

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| UnsetRepositoryNotificationChatError::DbError(e.to_string()))?;

        let repository = self
            .repository_repo
            .update(&txn, &repository)
            .await
            .map_err(|e| match e {
                UpdateRepositoryError::NotFound => UnsetRepositoryNotificationChatError::NotFound,
                UpdateRepositoryError::DbError(msg) => {
                    UnsetRepositoryNotificationChatError::DbError(msg)
                }
            })?;

        txn.commit()
            .await
            .map_err(|e| UnsetRepositoryNotificationChatError::DbError(e.to_string()))?;

        Ok(UnsetRepositoryNotificationChatResponse { repository })
    }
}
