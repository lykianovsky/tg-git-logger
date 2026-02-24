use crate::domain::task::ports::task_tracker_client::TaskTrackerClientMoveToColumnError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoveTaskToTestExecutorError {
    #[error("{0}")]
    TaskTrackerClientMoveToColumnError(#[from] TaskTrackerClientMoveToColumnError),
}
