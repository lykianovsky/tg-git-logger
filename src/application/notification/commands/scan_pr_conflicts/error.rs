use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanPrConflictsExecutorError {
    #[error("Database error: {0}")]
    DbError(String),
}
