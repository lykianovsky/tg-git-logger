use crate::application::test_run::queries::get_run_failures::error::GetRunFailuresError;
use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildTestReportError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Test run not found")]
    NotFound,

    #[error("APPLICATION_BASE_URL is not set")]
    BaseUrlNotConfigured,

    #[error("Render error: {0}")]
    RenderError(String),
}

impl From<FindTestRunError> for BuildTestReportError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NotFound,
        }
    }
}

impl From<GetRunFailuresError> for BuildTestReportError {
    fn from(error: GetRunFailuresError) -> Self {
        match error {
            GetRunFailuresError::DbError(message) => Self::DbError(message),
            GetRunFailuresError::NotFound => Self::NotFound,
        }
    }
}

impl From<FindRepositoryByIdError> for BuildTestReportError {
    fn from(error: FindRepositoryByIdError) -> Self {
        match error {
            FindRepositoryByIdError::DbError(message) => Self::DbError(message),
            FindRepositoryByIdError::NotFound => Self::NotFound,
        }
    }
}

impl From<askama::Error> for BuildTestReportError {
    fn from(error: askama::Error) -> Self {
        Self::RenderError(error.to_string())
    }
}
