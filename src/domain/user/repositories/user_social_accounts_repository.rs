use crate::domain::user::entities::user_social_account::UserSocialAccount;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use sea_orm::DatabaseTransaction;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateSocialServiceError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindSocialServiceByIdError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindSocialServiceByUserIdError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("User not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum FindSocialServiceByChatIdError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait UserSocialAccountsRepository: Send + Sync {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user_social: &UserSocialAccount,
    ) -> Result<UserSocialAccount, CreateSocialServiceError>;

    async fn find_by_social_user_id(
        &self,
        social_user_id: &SocialUserId,
    ) -> Result<UserSocialAccount, FindSocialServiceByIdError>;

    async fn find_by_user_id(
        &self,
        user_id: &crate::domain::user::value_objects::user_id::UserId,
    ) -> Result<UserSocialAccount, FindSocialServiceByUserIdError>;

    async fn find_by_social_chat_id(
        &self,
        social_chat_id: &SocialChatId,
        social_type: &SocialType,
    ) -> Result<Option<UserSocialAccount>, FindSocialServiceByChatIdError>;
}
