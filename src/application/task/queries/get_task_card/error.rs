use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetTaskCardError {
    #[error("Card not found")]
    NotFound,

    #[error("Task tracker error: {0}")]
    ClientError(String),
}
