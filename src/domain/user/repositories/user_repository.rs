use crate::domain::user::entities::user::User;
use crate::domain::user::value_objects::user_id::UserId;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateUserException {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindUserByIdException {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user: &User,
    ) -> Result<User, CreateUserException>;

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdException>;
}
