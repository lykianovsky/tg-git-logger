use crate::application::repository::commands::set_repository_notification_chat::command::SetRepositoryNotificationChatCommand;
use crate::application::repository::commands::set_repository_notification_chat::error::SetRepositoryNotificationChatError;
use crate::application::repository::commands::set_repository_notification_chat::response::SetRepositoryNotificationChatResponse;
use crate::domain::repository::repositories::repository_repository::{
    FindRepositoryByIdError, RepositoryRepository, UpdateRepositoryError,
};
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct SetRepositoryNotificationChatExecutor {
    pub db: Arc<DatabaseConnection>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
}

impl SetRepositoryNotificationChatExecutor {
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

impl CommandExecutor for SetRepositoryNotificationChatExecutor {
    type Command = SetRepositoryNotificationChatCommand;
    type Response = SetRepositoryNotificationChatResponse;
    type Error = SetRepositoryNotificationChatError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut repository = self
            .repository_repo
            .find_by_id(cmd.repository_id)
            .await
            .map_err(|e| match e {
                FindRepositoryByIdError::NotFound => SetRepositoryNotificationChatError::NotFound,
                FindRepositoryByIdError::DbError(msg) => {
                    SetRepositoryNotificationChatError::DbError(msg)
                }
            })?;

        repository.social_chat_id = Some(cmd.social_chat_id);
        repository.updated_at = Utc::now();

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| SetRepositoryNotificationChatError::DbError(e.to_string()))?;

        let repository = self
            .repository_repo
            .update(&txn, &repository)
            .await
            .map_err(|e| match e {
                UpdateRepositoryError::NotFound => SetRepositoryNotificationChatError::NotFound,
                UpdateRepositoryError::DbError(msg) => {
                    SetRepositoryNotificationChatError::DbError(msg)
                }
            })?;

        txn.commit()
            .await
            .map_err(|e| SetRepositoryNotificationChatError::DbError(e.to_string()))?;

        Ok(SetRepositoryNotificationChatResponse { repository })
    }
}
