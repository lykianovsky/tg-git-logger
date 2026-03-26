use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateRepositoryTaskTrackerExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
