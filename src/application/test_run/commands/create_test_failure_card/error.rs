use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::task::ports::task_tracker_client::{
    TaskTrackerClientCreateCardError, TaskTrackerClientListError,
};
use crate::domain::test_run::repositories::test_failure_card_repository::{
    FindTestFailureCardError, SaveTestFailureCardError,
};
use crate::domain::test_run::repositories::test_run_repository::FindTestRunError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateTestFailureCardError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Test failure not found")]
    NotFound,

    #[error("Task tracker is not configured for repository")]
    TrackerNotConfigured,

    #[error("Task tracker error: {0}")]
    TrackerError(String),
}

impl From<FindTestRunError> for CreateTestFailureCardError {
    fn from(error: FindTestRunError) -> Self {
        match error {
            FindTestRunError::DbError(message) => Self::DbError(message),
            FindTestRunError::NotFound => Self::NotFound,
        }
    }
}

impl From<FindRepositoryByIdError> for CreateTestFailureCardError {
    fn from(error: FindRepositoryByIdError) -> Self {
        match error {
            FindRepositoryByIdError::DbError(message) => Self::DbError(message),
            FindRepositoryByIdError::NotFound => Self::NotFound,
        }
    }
}

impl From<FindTestFailureCardError> for CreateTestFailureCardError {
    fn from(error: FindTestFailureCardError) -> Self {
        match error {
            FindTestFailureCardError::DbError(message) => Self::DbError(message),
        }
    }
}

impl From<SaveTestFailureCardError> for CreateTestFailureCardError {
    fn from(error: SaveTestFailureCardError) -> Self {
        match error {
            SaveTestFailureCardError::DbError(message) => Self::DbError(message),
        }
    }
}

impl From<TaskTrackerClientCreateCardError> for CreateTestFailureCardError {
    fn from(error: TaskTrackerClientCreateCardError) -> Self {
        match error {
            TaskTrackerClientCreateCardError::ClientError(message) => Self::TrackerError(message),
        }
    }
}

impl From<TaskTrackerClientListError> for CreateTestFailureCardError {
    fn from(error: TaskTrackerClientListError) -> Self {
        match error {
            TaskTrackerClientListError::ClientError(message) => Self::TrackerError(message),
        }
    }
}
