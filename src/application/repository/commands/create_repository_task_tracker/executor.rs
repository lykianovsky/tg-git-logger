use crate::application::repository::commands::create_repository_task_tracker::command::CreateRepositoryTaskTrackerCommand;
use crate::application::repository::commands::create_repository_task_tracker::error::CreateRepositoryTaskTrackerExecutorError;
use crate::application::repository::commands::create_repository_task_tracker::response::CreateRepositoryTaskTrackerResponse;
use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::repository::repositories::repository_task_tracker_repository::{
    CreateRepositoryTaskTrackerError, RepositoryTaskTrackerRepository,
};
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::DatabaseConnection;
use sea_orm::TransactionTrait;
use std::sync::Arc;

pub struct CreateRepositoryTaskTrackerExecutor {
    pub db: Arc<DatabaseConnection>,
    pub repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
}

impl CreateRepositoryTaskTrackerExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    ) -> Self {
        Self {
            db,
            repository_task_tracker_repo,
        }
    }
}

impl CommandExecutor for CreateRepositoryTaskTrackerExecutor {
    type Command = CreateRepositoryTaskTrackerCommand;
    type Response = CreateRepositoryTaskTrackerResponse;
    type Error = CreateRepositoryTaskTrackerExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| CreateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?;

        let placeholder = RepositoryTaskTracker {
            id: 0,
            repository_id: cmd.repository_id.clone(),
            space_id: cmd.space_id,
            qa_column_id: cmd.qa_column_id,
            extract_pattern_regexp: cmd.extract_pattern_regexp.clone(),
            path_to_card: cmd.path_to_card.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let tracker = self
            .repository_task_tracker_repo
            .create(&txn, &placeholder)
            .await
            .map_err(|e| match e {
                CreateRepositoryTaskTrackerError::DbError(msg) => {
                    CreateRepositoryTaskTrackerExecutorError::DbError(msg)
                }
            })?;

        txn.commit()
            .await
            .map_err(|e| CreateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?;

        Ok(CreateRepositoryTaskTrackerResponse { tracker })
    }
}
