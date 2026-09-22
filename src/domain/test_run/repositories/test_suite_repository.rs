use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_suite::TestSuite;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FindTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Tests are not configured for repository")]
    NotConfigured,
}

#[async_trait::async_trait]
pub trait TestSuiteRepository: Send + Sync {
    async fn find_by_repository(
        &self,
        repository_id: RepositoryId,
    ) -> Result<TestSuite, FindTestSuiteError>;
}
