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
pub enum TaskTrackerClientCreateCardError {
    #[error("{0}")]
    ClientError(String),
}

/// Новая карточка: доска и колонка берутся из настроек трекера репозитория,
/// ответственного и тег выбирает человек в боте
pub struct NewTaskTrackerCard {
    pub board_id: i32,
    pub column_id: i32,
    pub title: String,
    pub description: String,
    pub responsible_id: Option<u64>,
    /// Тег привязывается по названию: в Kaiten оно и есть идентификатор тега карточки
    pub tag: Option<String>,
}

pub struct TaskTrackerUser {
    pub id: u64,
    pub name: String,
}

pub struct TaskTrackerTag {
    pub id: u64,
    pub name: String,
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

    async fn create_card(
        &self,
        card: &NewTaskTrackerCard,
    ) -> Result<TaskTrackerCard, TaskTrackerClientCreateCardError>;

    async fn list_users(&self) -> Result<Vec<TaskTrackerUser>, TaskTrackerClientListError>;

    async fn list_tags(&self) -> Result<Vec<TaskTrackerTag>, TaskTrackerClientListError>;

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
