use crate::domain::user::entities::user_vcs::UserVersionControlService;
use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user::value_objects::version_control_user_id::VersionControlUserId;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateVersionControlServiceException {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindVersionControlServiceByIdException {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindVersionControlServiceByUserIdException {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait UserVersionControlServicesRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user: &UserVersionControlService,
    ) -> Result<UserVersionControlService, CreateVersionControlServiceException>;

    async fn find_by_version_control_user_id(
        &self,
        id: &VersionControlUserId,
    ) -> Result<UserVersionControlService, FindVersionControlServiceByIdException>;

    async fn find_by_user_id(
        &self,
        id: &UserId,
    ) -> Result<UserVersionControlService, FindVersionControlServiceByUserIdException>;
}
