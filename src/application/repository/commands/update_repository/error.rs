use crate::domain::repository::repositories::repository_repository::{
    FindRepositoryByIdError, UpdateRepositoryError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateRepositoryExecutorError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("{0}")]
    NotFound(#[from] FindRepositoryByIdError),

    #[error("{0}")]
    UpdateError(#[from] UpdateRepositoryError),
}
