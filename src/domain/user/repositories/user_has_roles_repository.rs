use crate::domain::role::entities::role_entity::Role;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user::value_objects::user_id::UserId;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssignRoleToUserException {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum GetAllUserRolesException {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Invalid field: {0}")]
    InvalidField(String),
}

#[async_trait::async_trait]
pub trait UserHasRolesRepository: Send + Sync {
    async fn assign(
        &self,
        txn: &DatabaseTransaction,
        user_id: UserId,
        role_name: RoleName,
    ) -> Result<(), AssignRoleToUserException>;

    async fn get_all(&self, user_id: UserId) -> Result<Vec<Role>, GetAllUserRolesException>;
}
