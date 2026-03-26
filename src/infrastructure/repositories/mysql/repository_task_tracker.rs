use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::repository::repositories::repository_task_tracker_repository::{
    CreateRepositoryTaskTrackerError, FindRepositoryTaskTrackerByIdError,
    FindRepositoryTaskTrackerByRepositoryIdError, RepositoryTaskTrackerRepository,
    UpdateRepositoryTaskTrackerError,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::infrastructure::database::mysql::entities::repository_task_tracker;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::sync::Arc;

pub struct MySQLRepositoryTaskTrackerRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLRepositoryTaskTrackerRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RepositoryTaskTrackerRepository for MySQLRepositoryTaskTrackerRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        tracker: &RepositoryTaskTracker,
    ) -> Result<RepositoryTaskTracker, CreateRepositoryTaskTrackerError> {
        let model = repository_task_tracker::ActiveModel {
            repository_id: Set(tracker.repository_id.0),
            space_id: Set(tracker.space_id),
            qa_column_id: Set(tracker.qa_column_id),
            extract_pattern_regexp: Set(tracker.extract_pattern_regexp.clone()),
            path_to_card: Set(tracker.path_to_card.clone()),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateRepositoryTaskTrackerError::DbError(e.to_string()))?;

        Ok(RepositoryTaskTracker::from_mysql(result))
    }

    async fn update(
        &self,
        txn: &DatabaseTransaction,
        tracker: &RepositoryTaskTracker,
    ) -> Result<RepositoryTaskTracker, UpdateRepositoryTaskTrackerError> {
        let model = repository_task_tracker::ActiveModel {
            id: Set(tracker.id),
            repository_id: Set(tracker.repository_id.0),
            space_id: Set(tracker.space_id),
            qa_column_id: Set(tracker.qa_column_id),
            extract_pattern_regexp: Set(tracker.extract_pattern_regexp.clone()),
            path_to_card: Set(tracker.path_to_card.clone()),
            ..Default::default()
        };

        let result = model
            .update(txn)
            .await
            .map_err(|e| UpdateRepositoryTaskTrackerError::DbError(e.to_string()))?;

        Ok(RepositoryTaskTracker::from_mysql(result))
    }

    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<RepositoryTaskTracker, FindRepositoryTaskTrackerByIdError> {
        let result = repository_task_tracker::Entity::find()
            .filter(repository_task_tracker::Column::Id.eq(id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRepositoryTaskTrackerByIdError::DbError(e.to_string()))?
            .ok_or(FindRepositoryTaskTrackerByIdError::NotFound)?;

        Ok(RepositoryTaskTracker::from_mysql(result))
    }

    async fn find_by_repository_id(
        &self,
        repository_id: RepositoryId,
    ) -> Result<RepositoryTaskTracker, FindRepositoryTaskTrackerByRepositoryIdError> {
        let result = repository_task_tracker::Entity::find()
            .filter(repository_task_tracker::Column::RepositoryId.eq(repository_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRepositoryTaskTrackerByRepositoryIdError::DbError(e.to_string()))?
            .ok_or(FindRepositoryTaskTrackerByRepositoryIdError::NotFound)?;

        Ok(RepositoryTaskTracker::from_mysql(result))
    }
}

impl RepositoryTaskTracker {
    pub fn from_mysql(model: repository_task_tracker::Model) -> Self {
        Self {
            id: model.id,
            repository_id: RepositoryId(model.repository_id),
            space_id: model.space_id,
            qa_column_id: model.qa_column_id,
            extract_pattern_regexp: model.extract_pattern_regexp,
            path_to_card: model.path_to_card,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
