use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateRepositoryTaskTrackerExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
