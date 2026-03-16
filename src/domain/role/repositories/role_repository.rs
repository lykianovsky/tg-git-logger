use crate::domain::role::entities::role_entity::Role;
use crate::domain::role::value_objects::role_id::RoleId;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateRoleError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindRoleByIdError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Role not found")]
    NotFound,
}

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn create(&self, txn: &DatabaseTransaction, role: &Role) -> Result<(), CreateRoleError>;
    async fn find_by_id(&self, id: RoleId) -> Result<Role, FindRoleByIdError>;
}
