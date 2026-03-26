use crate::application::repository::commands::update_repository_task_tracker::command::UpdateRepositoryTaskTrackerCommand;
use crate::application::repository::commands::update_repository_task_tracker::error::UpdateRepositoryTaskTrackerExecutorError;
use crate::application::repository::commands::update_repository_task_tracker::response::UpdateRepositoryTaskTrackerResponse;
use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::repository::repositories::repository_task_tracker_repository::{
    FindRepositoryTaskTrackerByRepositoryIdError, RepositoryTaskTrackerRepository,
};
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct UpdateRepositoryTaskTrackerExecutor {
    db: Arc<DatabaseConnection>,
    task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
}

impl UpdateRepositoryTaskTrackerExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    ) -> Self {
        Self {
            db,
            task_tracker_repo,
        }
    }
}

impl CommandExecutor for UpdateRepositoryTaskTrackerExecutor {
    type Command = UpdateRepositoryTaskTrackerCommand;
    type Response = UpdateRepositoryTaskTrackerResponse;
    type Error = UpdateRepositoryTaskTrackerExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| UpdateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?;

        let existing = self
            .task_tracker_repo
            .find_by_repository_id(cmd.repository_id)
            .await;

        let tracker = match existing {
            Ok(mut tracker) => {
                tracker.space_id = cmd.space_id;
                tracker.qa_column_id = cmd.qa_column_id;
                tracker.extract_pattern_regexp = cmd.extract_pattern_regexp.clone();
                tracker.path_to_card = cmd.path_to_card.clone();
                tracker.updated_at = Utc::now();
                self.task_tracker_repo
                    .update(&txn, &tracker)
                    .await
                    .map_err(|e| UpdateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?
            }
            Err(FindRepositoryTaskTrackerByRepositoryIdError::NotFound) => {
                let placeholder = RepositoryTaskTracker {
                    id: Default::default(),
                    repository_id: cmd.repository_id,
                    space_id: cmd.space_id,
                    qa_column_id: cmd.qa_column_id,
                    extract_pattern_regexp: cmd.extract_pattern_regexp.clone(),
                    path_to_card: cmd.path_to_card.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                self.task_tracker_repo
                    .create(&txn, &placeholder)
                    .await
                    .map_err(|e| UpdateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?
            }
            Err(e) => {
                return Err(UpdateRepositoryTaskTrackerExecutorError::DbError(
                    e.to_string(),
                ));
            }
        };

        txn.commit()
            .await
            .map_err(|e| UpdateRepositoryTaskTrackerExecutorError::DbError(e.to_string()))?;

        Ok(UpdateRepositoryTaskTrackerResponse { tracker })
    }
}
