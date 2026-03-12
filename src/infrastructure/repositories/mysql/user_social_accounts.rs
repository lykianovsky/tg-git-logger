use crate::domain::user::entities::user_social_account::UserSocialAccount;
use crate::domain::user::repositories::user_social_accounts_repository::{
    CreateSocialServiceError, FindSocialServiceByIdError, UserSocialAccountsRepository,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::user_social_accounts;
use sea_orm::{ActiveModelTrait, DatabaseTransaction};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{DatabaseConnection, Set};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLUserSocialServicesRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserSocialServicesRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserSocialAccountsRepository for MySQLUserSocialServicesRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user_social: &UserSocialAccount,
    ) -> Result<UserSocialAccount, CreateSocialServiceError> {
        let model = user_social_accounts::ActiveModel {
            id: Default::default(),
            user_id: Set(user_social.user_id.0),
            social_type: Set(user_social.social_type.to_string()),
            social_user_id: Set(user_social.social_user_id.0 as i64),
            social_chat_id: Set(user_social.social_chat_id.0),
            social_user_login: Set(user_social.social_user_login.clone()),
            social_user_avatar_url: Set(user_social.social_user_avatar_url.clone()),
            social_user_email: Set(user_social.social_user_email.clone()),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateSocialServiceError::DbError(e.to_string()))?;

        Ok(UserSocialAccount::from_mysql(result).map_err(CreateSocialServiceError::DbError)?)
    }

    async fn find_by_social_user_id(
        &self,
        social_user_id: &SocialUserId,
    ) -> Result<UserSocialAccount, FindSocialServiceByIdError> {
        let user = user_social_accounts::Entity::find()
            .filter(user_social_accounts::Column::SocialUserId.eq(social_user_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindSocialServiceByIdError::DbError(e.to_string()))?
            .ok_or(FindSocialServiceByIdError::NotFound)?;

        Ok(UserSocialAccount::from_mysql(user).map_err(FindSocialServiceByIdError::DbError)?)
    }
}

impl UserSocialAccount {
    pub fn from_mysql(mysql_user: user_social_accounts::Model) -> Result<Self, String> {
        let social_type = SocialType::from_str(&mysql_user.social_type).map_err(|e| {
            format!(
                "Invalid social type in DB: {}, error: {:?}",
                mysql_user.social_type, e
            )
        })?;

        Ok(Self {
            id: mysql_user.id,
            user_id: UserId(mysql_user.user_id),
            social_type,
            social_user_id: SocialUserId(mysql_user.social_user_id as i32),
            social_chat_id: SocialChatId(mysql_user.social_chat_id),
            social_user_login: mysql_user.social_user_login,
            social_user_email: mysql_user.social_user_email,
            social_user_avatar_url: mysql_user.social_user_avatar_url,
            created_at: mysql_user.created_at,
            updated_at: mysql_user.updated_at,
        })
    }
}
