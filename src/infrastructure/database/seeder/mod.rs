use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SeederRunError {
    #[error("Failed to insert: {0}")]
    DbError(String),

    #[error("Failed to insert: {0}")]
    InsertFailed(String),
}

#[async_trait]
pub trait Seeder {
    async fn run(self) -> Result<(), SeederRunError>;
}
