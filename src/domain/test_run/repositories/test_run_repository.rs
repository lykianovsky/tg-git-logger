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

    /// Прогоны, по которым не пришёл вебхук, — их статус добирает планировщик
    async fn find_stale_active(&self, limit: u64) -> Result<Vec<TestRun>, FindTestRunError>;

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

    async fn list_failures(&self, id: TestRunId) -> Result<Vec<TestFailure>, FindTestRunError>;
}
