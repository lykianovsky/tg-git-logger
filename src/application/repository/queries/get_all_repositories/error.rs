use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetAllRepositoriesError {
    #[error("Database error: {0}")]
    DbError(String),
}
