use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildQualityDashboardError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Repository not found")]
    NotFound,

    #[error("APPLICATION_BASE_URL is not set")]
    BaseUrlNotConfigured,

    #[error("Render error: {0}")]
    RenderError(String),
}

impl From<FindTestRunError> for BuildQualityDashboardError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NotFound,
        }
    }
}

impl From<FindRepositoryByIdError> for BuildQualityDashboardError {
    fn from(error: FindRepositoryByIdError) -> Self {
        match error {
            FindRepositoryByIdError::DbError(message) => Self::DbError(message),
            FindRepositoryByIdError::NotFound => Self::NotFound,
        }
    }
}

impl From<askama::Error> for BuildQualityDashboardError {
    fn from(error: askama::Error) -> Self {
        Self::RenderError(error.to_string())
    }
}
