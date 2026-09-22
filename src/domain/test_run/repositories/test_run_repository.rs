use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_failure::{NewTestFailure, TestFailure};
use crate::domain::test_run::entities::test_run::{NewTestRun, TestRun, TestRunOutcome};
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateTestRunError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindTestRunError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Test run not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateTestRunError {
    #[error("Database error: {0}")]
    DbError(String),
}

/// Сколько раз тест падал за период
#[derive(Debug, Clone)]
pub struct TestFailureCount {
    pub project: String,
    pub file: String,
    pub title: String,
    pub count: u32,
}

#[async_trait::async_trait]
pub trait TestRunRepository: Send + Sync {
    async fn create(&self, run: &NewTestRun) -> Result<TestRun, CreateTestRunError>;

    async fn find_by_id(&self, id: TestRunId) -> Result<TestRun, FindTestRunError>;

    async fn find_by_tag(&self, tag: &RunTag) -> Result<TestRun, FindTestRunError>;

    /// Последний прогон репозитория — его показывает карточка в чате
    async fn find_last(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<TestRun>, FindTestRunError>;

    /// Незавершённый прогон: пока он есть, второй запуск не отправляется
    async fn find_active(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<TestRun>, FindTestRunError>;

    /// История прогонов репозитория: на ней строится дашборд качества
    async fn list_recent(
        &self,
        repository_id: RepositoryId,
        limit: u64,
    ) -> Result<Vec<TestRun>, FindTestRunError>;

    /// Падения за период, сгруппированные по тесту: показывают, что падает постоянно
    async fn count_failures_since(
        &self,
        repository_id: RepositoryId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<TestFailureCount>, FindTestRunError>;

    /// Все идущие прогоны: планировщик держит их карточки в актуальном состоянии
    async fn find_all_active(&self, limit: u64) -> Result<Vec<TestRun>, FindTestRunError>;

    async fn mark_started(
        &self,
        id: TestRunId,
        provider_run_id: u64,
        run_url: String,
    ) -> Result<(), UpdateTestRunError>;

    async fn save_outcome(
        &self,
        id: TestRunId,
        outcome: &TestRunOutcome,
    ) -> Result<(), UpdateTestRunError>;

    async fn attach_message(
        &self,
        id: TestRunId,
        chat_id: SocialChatId,
        message_id: i32,
    ) -> Result<(), UpdateTestRunError>;

    async fn replace_failures(
        &self,
        id: TestRunId,
        failures: &[NewTestFailure],
    ) -> Result<(), UpdateTestRunError>;

    async fn find_failure(&self, id: i32) -> Result<TestFailure, FindTestRunError>;

    async fn list_failures(&self, id: TestRunId) -> Result<Vec<TestFailure>, FindTestRunError>;
}
