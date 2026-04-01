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

#[derive(Error, Debug)]
pub enum TaskTrackerClientListError {
    #[error("{0}")]
    ClientError(String),
}

pub struct TaskTrackerCard {
    pub id: TaskId,
    pub title: String,
    pub url: String,
}

pub struct TaskTrackerSpace {
    pub id: i32,
    pub title: String,
}

pub struct TaskTrackerBoard {
    pub id: i32,
    pub title: String,
}

pub struct TaskTrackerColumn {
    pub id: i32,
    pub title: String,
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

    async fn list_spaces(&self) -> Result<Vec<TaskTrackerSpace>, TaskTrackerClientListError>;

    async fn list_boards(
        &self,
        space_id: i32,
    ) -> Result<Vec<TaskTrackerBoard>, TaskTrackerClientListError>;

    async fn list_columns(
        &self,
        board_id: i32,
    ) -> Result<Vec<TaskTrackerColumn>, TaskTrackerClientListError>;
}
