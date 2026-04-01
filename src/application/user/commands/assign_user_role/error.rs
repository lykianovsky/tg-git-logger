use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssignUserRoleExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
