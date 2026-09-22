use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetTestSuiteError {
    #[error("Database error: {0}")]
    DbError(String),
}
