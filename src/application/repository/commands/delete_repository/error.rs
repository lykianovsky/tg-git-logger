use crate::domain::repository::repositories::repository_repository::{
    DeleteRepositoryError, FindRepositoryByIdError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteRepositoryExecutorError {
    #[error("{0}")]
    FindRepositoryByIdError(#[from] FindRepositoryByIdError),

    #[error("{0}")]
    DeleteRepositoryError(#[from] DeleteRepositoryError),

    #[error("Database error: {0}")]
    DbError(String),
}
