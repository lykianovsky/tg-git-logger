use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateRepositoryError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum UpdateRepositoryError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindRepositoryByIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindRepositoryByExternalIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindAllRepositoriesError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum DeleteRepositoryError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Repository not found")]
    NotFound,
}

#[async_trait]
pub trait RepositoryRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        repository: &Repository,
    ) -> Result<Repository, CreateRepositoryError>;

    async fn update(
        &self,
        txn: &DatabaseTransaction,
        repository: &Repository,
    ) -> Result<Repository, UpdateRepositoryError>;

    async fn find_by_id(&self, id: RepositoryId) -> Result<Repository, FindRepositoryByIdError>;

    async fn find_by_external_id(
        &self,
        external_id: i64,
    ) -> Result<Repository, FindRepositoryByExternalIdError>;

    async fn find_all(&self) -> Result<Vec<Repository>, FindAllRepositoriesError>;

    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        id: RepositoryId,
    ) -> Result<(), DeleteRepositoryError>;
}
