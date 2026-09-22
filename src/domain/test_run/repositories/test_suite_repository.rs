use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_suite::{NewTestSuite, TestSuite};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FindTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Tests are not configured for repository")]
    NotConfigured,
}

#[derive(Debug, Error)]
pub enum SaveTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait TestSuiteRepository: Send + Sync {
    async fn find_by_repository(
        &self,
        repository_id: RepositoryId,
    ) -> Result<TestSuite, FindTestSuiteError>;

    /// Повторное подключение переписывает настройки того же репозитория
    async fn upsert(&self, suite: &NewTestSuite) -> Result<(), SaveTestSuiteError>;

    /// Отключение тестов: прогоны репозитория остаются в истории
    async fn delete(&self, repository_id: RepositoryId) -> Result<(), SaveTestSuiteError>;
}
