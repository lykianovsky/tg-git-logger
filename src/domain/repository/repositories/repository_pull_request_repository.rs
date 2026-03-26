use crate::domain::repository::entities::repository_pull_request::RepositoryPullRequest;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreatePullRequestError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindPullRequestByIdError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Pull request not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindPullRequestsByRepositoryIdError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait]
pub trait RepositoryPullRequestRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        pull_request: &RepositoryPullRequest,
    ) -> Result<RepositoryPullRequest, CreatePullRequestError>;

    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<RepositoryPullRequest, FindPullRequestByIdError>;

    async fn find_by_repository_id(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Vec<RepositoryPullRequest>, FindPullRequestsByRepositoryIdError>;
}
