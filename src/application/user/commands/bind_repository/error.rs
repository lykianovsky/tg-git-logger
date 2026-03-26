use crate::domain::repository::repositories::repository_repository::FindRepositoryByIdError;
use crate::domain::user::repositories::user_connection_repositories_repository::{
    CreateUserConnectionRepositoryError, FindUserConnectionRepositoryByUserAndRepoError,
};
use crate::domain::user::repositories::user_social_accounts_repository::FindSocialServiceByIdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindRepositoryExecutorError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("{0}")]
    UserNotFound(#[from] FindSocialServiceByIdError),

    #[error("{0}")]
    RepositoryNotFound(#[from] FindRepositoryByIdError),

    #[error("{0}")]
    LookupError(#[from] FindUserConnectionRepositoryByUserAndRepoError),

    #[error("{0}")]
    CreateError(#[from] CreateUserConnectionRepositoryError),

    #[error("Already bound to this repository")]
    AlreadyBound,
}
