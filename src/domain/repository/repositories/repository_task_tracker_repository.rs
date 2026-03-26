use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateRepositoryTaskTrackerError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindRepositoryTaskTrackerByIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository task tracker not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindRepositoryTaskTrackerByRepositoryIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository task tracker not found")]
    NotFound,
}

#[async_trait]
pub trait RepositoryTaskTrackerRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        tracker: &RepositoryTaskTracker,
    ) -> Result<RepositoryTaskTracker, CreateRepositoryTaskTrackerError>;

    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<RepositoryTaskTracker, FindRepositoryTaskTrackerByIdError>;

    async fn find_by_repository_id(
        &self,
        repository_id: RepositoryId,
    ) -> Result<RepositoryTaskTracker, FindRepositoryTaskTrackerByRepositoryIdError>;
}
