use thiserror::Error;

#[derive(Debug, Error)]
pub enum RemoveUserRoleExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
