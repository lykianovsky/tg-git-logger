use crate::domain::test_run::entities::test_run::{TestRunOutcome, TestRunTotals};
use crate::domain::test_run::entities::test_suite::TestSuite;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchTestRunError {
    #[error("CI error: {0}")]
    ProviderError(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
}

#[derive(Debug, Error)]
pub enum FetchTestRunError {
    #[error("CI error: {0}")]
    ProviderError(String),

    #[error("Run not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum ListTestBlocksError {
    #[error("CI error: {0}")]
    ProviderError(String),
}

/// Итоги прогона, разобранные из артефакта: сам файл читает адаптер CI
#[derive(Debug, Clone)]
pub struct TestRunArtifacts {
    pub totals: Option<TestRunTotals>,
    pub failures: Vec<ParsedTestFailure>,
}

#[derive(Debug, Clone)]
pub struct ParsedTestFailure {
    pub project: String,
    pub file: String,
    pub title: String,
    pub error_excerpt: Option<String>,
}

#[async_trait::async_trait]
pub trait TestRunner: Send + Sync {
    /// Отправляет запуск в CI. Идентификатор прогона появится позже — его находят по метке.
    /// Токен передаётся вызовом: в CI ходим правами того, кто запустил прогон
    async fn dispatch(
        &self,
        token: &str,
        suite: &TestSuite,
        git_ref: &str,
        args: &str,
        tag: &RunTag,
    ) -> Result<(), DispatchTestRunError>;

    async fn find_run_by_tag(
        &self,
        token: &str,
        suite: &TestSuite,
        tag: &RunTag,
    ) -> Result<TestRunOutcome, FetchTestRunError>;

    async fn fetch_artifacts(
        &self,
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<TestRunArtifacts, FetchTestRunError>;

    /// Блоки тестов — каталоги внутри `tests_root` тестируемой ветки
    async fn list_blocks(
        &self,
        token: &str,
        suite: &TestSuite,
    ) -> Result<Vec<String>, ListTestBlocksError>;
}
