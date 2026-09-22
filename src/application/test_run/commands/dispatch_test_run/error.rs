use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::ports::test_runner::DispatchTestRunError as RunnerDispatchError;
use crate::domain::test_run::repositories::test_run_repository::{
    CreateTestRunError, FindTestRunError,
};
use crate::domain::test_run::repositories::test_suite_repository::FindTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DispatchTestRunError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Tests are not configured for repository")]
    NotConfigured,

    /// Прогон по репозиторию уже идёт — второй не запускаем, показываем текущий
    #[error("Test run is already in progress")]
    AlreadyRunning(Box<TestRun>),

    #[error("CI error: {0}")]
    ProviderError(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
}

impl From<FindTestSuiteError> for DispatchTestRunError {
    fn from(error: FindTestSuiteError) -> Self {
        match error {
            FindTestSuiteError::DbError(message) => Self::DbError(message),
            FindTestSuiteError::NotConfigured => Self::NotConfigured,
        }
    }
}

impl From<FindTestRunError> for DispatchTestRunError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::DbError("Test run not found".to_string()),
        }
    }
}

impl From<CreateTestRunError> for DispatchTestRunError {
    fn from(error: CreateTestRunError) -> Self {
        match error {
            CreateTestRunError::DbError(message) => Self::DbError(message),
        }
    }
}

impl From<RunnerDispatchError> for DispatchTestRunError {
    fn from(error: RunnerDispatchError) -> Self {
        match error {
            RunnerDispatchError::ProviderError(message) => Self::ProviderError(message),
            RunnerDispatchError::WorkflowNotFound(file) => Self::WorkflowNotFound(file),
        }
    }
}
