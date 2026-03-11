use crate::domain::task::value_objects::task_id::TaskId;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskTrackerClientMoveToColumnError {
    #[error("Card move validation failed")]
    MoveValidationFailed,

    #[error("{0}")]
    ClientError(String),

    #[error("{0}")]
    ParseError(String),
}

#[async_trait]
pub trait TaskTrackerClient: Send + Sync {
    async fn move_task_to_column(
        &self,
        task_id: TaskId,
        column_id: u64,
    ) -> Result<(), TaskTrackerClientMoveToColumnError>;
}
