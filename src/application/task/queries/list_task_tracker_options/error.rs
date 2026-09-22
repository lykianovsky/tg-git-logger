use crate::domain::task::ports::task_tracker_client::TaskTrackerClientListError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListTaskTrackerOptionsError {
    #[error("Task tracker error: {0}")]
    TrackerError(String),
}

impl From<TaskTrackerClientListError> for ListTaskTrackerOptionsError {
    fn from(error: TaskTrackerClientListError) -> Self {
        match error {
            TaskTrackerClientListError::ClientError(message) => Self::TrackerError(message),
        }
    }
}
