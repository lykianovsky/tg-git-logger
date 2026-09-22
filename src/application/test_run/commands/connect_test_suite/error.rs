use crate::domain::test_run::repositories::test_suite_repository::SaveTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Workflow file is empty")]
    EmptyWorkflowFile,

    #[error("Default ref is empty")]
    EmptyDefaultRef,
}

impl From<SaveTestSuiteError> for ConnectTestSuiteError {
    fn from(error: SaveTestSuiteError) -> Self {
        match error {
            SaveTestSuiteError::DbError(message) => Self::DbError(message),
        }
    }
}
