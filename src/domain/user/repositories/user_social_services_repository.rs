use crate::domain::user::entities::user_social::UserSocial;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateSocialServiceException {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindSocialServiceByIdException {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[async_trait::async_trait]
pub trait UserSocialServicesRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user_social: &UserSocial,
    ) -> Result<UserSocial, CreateSocialServiceException>;

    async fn find_by_social_user_id(
        &self,
        social_user_id: &SocialUserId,
    ) -> Result<UserSocial, FindSocialServiceByIdException>;
}
