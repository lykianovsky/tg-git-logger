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

#[derive(Error, Debug)]
pub enum TaskTrackerClientGetCardError {
    #[error("Card not found")]
    NotFound,

    #[error("{0}")]
    ClientError(String),
}

pub struct TaskTrackerCard {
    pub id: TaskId,
    pub title: String,
    pub url: String,
}

#[async_trait]
pub trait TaskTrackerClient: Send + Sync {
    async fn move_task_to_column(
        &self,
        task_id: TaskId,
        column_id: u64,
    ) -> Result<(), TaskTrackerClientMoveToColumnError>;

    async fn get_card(
        &self,
        task_id: TaskId,
    ) -> Result<TaskTrackerCard, TaskTrackerClientGetCardError>;
}
