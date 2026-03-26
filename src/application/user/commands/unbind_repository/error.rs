use crate::domain::user::repositories::user_connection_repositories_repository::DeleteUserConnectionRepositoryError;
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnbindRepositoryExecutorError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("{0}")]
    UserNotFound(#[from] FindSocialServiceByIdError),

    #[error("{0}")]
    DeleteError(#[from] DeleteUserConnectionRepositoryError),
}
