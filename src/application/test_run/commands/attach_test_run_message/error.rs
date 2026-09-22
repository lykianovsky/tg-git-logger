use crate::domain::test_run::repositories::test_run_repository::UpdateTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttachTestRunMessageError {
    #[error("Database error: {0}")]
    DbError(String),
}

impl From<UpdateTestRunError> for AttachTestRunMessageError {
    fn from(error: UpdateTestRunError) -> Self {
        match error {
            UpdateTestRunError::DbError(message) => Self::DbError(message),
        }
    }
}
