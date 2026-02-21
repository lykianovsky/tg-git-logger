use crate::domain::user::entities::user_social::UserSocial;
use crate::domain::user::repositories::user_social_services_repository::{
    CreateSocialServiceException, FindSocialServiceByIdException, UserSocialServicesRepository,
};
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::user_socials_services;
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
impl UserSocialServicesRepository for MySQLUserSocialServicesRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user_social: &UserSocial,
    ) -> Result<UserSocial, CreateSocialServiceException> {
        let model = user_socials_services::ActiveModel {
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
            .map_err(|e| CreateSocialServiceException::DbError(e.to_string()))?;

        Ok(UserSocial::from_mysql(result))
    }

    async fn find_by_social_user_id(
        &self,
        social_user_id: &SocialUserId,
    ) -> Result<UserSocial, FindSocialServiceByIdException> {
        let user = user_socials_services::Entity::find()
            .filter(user_socials_services::Column::SocialUserId.eq(social_user_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindSocialServiceByIdException::DbError(e.to_string()))?
            .ok_or(FindSocialServiceByIdException::NotFound)?;

        Ok(UserSocial::from_mysql(user))
    }
}

impl UserSocial {
    pub fn from_mysql(mysql_user: user_socials_services::Model) -> Self {
        Self {
            id: mysql_user.id,
            user_id: UserId(mysql_user.user_id),
            social_type: SocialType::from_str(&mysql_user.social_type).unwrap(),
            social_user_id: SocialUserId(mysql_user.social_user_id as i32),
            social_chat_id: SocialChatId(mysql_user.social_chat_id),
            social_user_login: mysql_user.social_user_login,
            social_user_email: mysql_user.social_user_email,
            social_user_avatar_url: mysql_user.social_user_avatar_url,
            created_at: mysql_user.created_at,
            updated_at: mysql_user.updated_at,
        }
    }
}
