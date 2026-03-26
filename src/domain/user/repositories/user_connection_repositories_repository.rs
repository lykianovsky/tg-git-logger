use crate::domain::user::entities::user_connection_repository::UserConnectionRepository;
use crate::domain::user::value_objects::user_id::UserId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateUserConnectionRepositoryError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindUserConnectionRepositoriesByUserIdError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait]
pub trait UserConnectionRepositoriesRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        connection: &UserConnectionRepository,
    ) -> Result<UserConnectionRepository, CreateUserConnectionRepositoryError>;

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<UserConnectionRepository>, FindUserConnectionRepositoriesByUserIdError>;
}
