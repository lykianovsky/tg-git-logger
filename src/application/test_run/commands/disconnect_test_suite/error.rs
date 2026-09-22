use crate::domain::test_run::repositories::test_suite_repository::SaveTestSuiteError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisconnectTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<SaveTestSuiteError> for DisconnectTestSuiteError {
    fn from(error: SaveTestSuiteError) -> Self {
        match error {
            SaveTestSuiteError::DbError(message) => Self::DbError(message),
        }
    }
}
