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

/// Насколько прогон продвинулся: CI сообщает шаги, а не отдельные тесты
#[derive(Debug, Clone)]
pub struct TestRunProgress {
    pub completed_steps: u32,
    pub total_steps: u32,
    /// Шаг, который выполняется прямо сейчас
    pub current_step: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedTestFailure {
    pub project: String,
    pub file: String,
    pub title: String,
    pub error_excerpt: Option<String>,
}

/// Вариант для подключения тестов: процесс CI или ветка репозитория
#[derive(Debug, Clone)]
pub struct CiOption {
    /// Значение, которое сохраняем (файл workflow или имя ветки)
    pub value: String,
    /// Подпись на кнопке
    pub label: String,
}

#[async_trait::async_trait]
pub trait TestRunner: Send + Sync {
    /// Процессы CI репозитория — из них человек выбирает тот, что гоняет тесты
    async fn list_workflows(
        &self,
        token: &str,
        owner: &str,
        name: &str,
    ) -> Result<Vec<CiOption>, ListTestBlocksError>;

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

    /// Ход прогона: по нему карточка показывает полосу прогресса
    async fn fetch_progress(
        &self,
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<Option<TestRunProgress>, FetchTestRunError>;

    /// Отмена прогона в CI: запустили не то — не ждём весь прогон
    async fn cancel_run(
        &self,
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<(), DispatchTestRunError>;

    /// Блоки тестов — каталоги внутри `tests_root` тестируемой ветки
    async fn list_blocks(
        &self,
        token: &str,
        suite: &TestSuite,
    ) -> Result<Vec<String>, ListTestBlocksError>;
}
