use crate::domain::user::entities::user_vcs::UserVersionControlService;
use crate::domain::user::repositories::user_version_control_services::{
    CreateVersionControlServiceException, FindVersionControlServiceByIdException,
    FindVersionControlServiceByUserIdException, UserVersionControlServicesRepository,
};
use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use crate::domain::user::value_objects::version_control_user_id::VersionControlUserId;
use crate::infrastructure::database::mysql::entities::user_version_control_services;
use crate::utils::security::crypto::reversible::ReversibleCipherValue;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLUserVersionControlServicesRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserVersionControlServicesRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UserVersionControlServicesRepository for MySQLUserVersionControlServicesRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user: &UserVersionControlService,
    ) -> Result<UserVersionControlService, CreateVersionControlServiceException> {
        let model = user_version_control_services::ActiveModel {
            user_id: Set(user.user_id.0),
            version_control_type: Set(user.version_control_type.to_string()),
            version_control_user_id: Set(user.version_control_user_id.0 as i64),
            version_control_login: Set(user.version_control_login.clone()),
            version_control_email: Set(user.version_control_email.clone()),
            version_control_avatar_url: Set(user.version_control_avatar_url.clone()),
            access_token: Set(user.access_token.value().to_string()),
            refresh_token: Set(user.refresh_token.clone()),
            token_type: Set(user.token_type.clone()),
            expires_at: Set(user.expires_at),
            scope: Set(user.scope.clone()),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateVersionControlServiceException::DbError(e.to_string()))?;

        UserVersionControlService::from_mysql(result)
            .map_err(|e| CreateVersionControlServiceException::DbError(e))
    }

    async fn find_by_version_control_user_id(
        &self,
        id: &VersionControlUserId,
    ) -> Result<UserVersionControlService, FindVersionControlServiceByIdException> {
        let result = user_version_control_services::Entity::find()
            .filter(user_version_control_services::Column::VersionControlUserId.eq(id.0 as i64))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindVersionControlServiceByIdException::DbError(e.to_string()))?
            .ok_or(FindVersionControlServiceByIdException::NotFound)?;

        UserVersionControlService::from_mysql(result)
            .map_err(|e| FindVersionControlServiceByIdException::DbError(e))
    }

    async fn find_by_user_id(
        &self,
        id: &UserId,
    ) -> Result<UserVersionControlService, FindVersionControlServiceByUserIdException> {
        let result = user_version_control_services::Entity::find()
            .filter(user_version_control_services::Column::UserId.eq(id.0 as i64))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindVersionControlServiceByUserIdException::DbError(e.to_string()))?
            .ok_or(FindVersionControlServiceByUserIdException::NotFound)?;

        UserVersionControlService::from_mysql(result)
            .map_err(|e| FindVersionControlServiceByUserIdException::DbError(e))
    }
}

impl UserVersionControlService {
    pub fn from_mysql(model: user_version_control_services::Model) -> Result<Self, String> {
        let version_control_type = VersionControlType::from_str(&model.version_control_type)
            .map_err(|e| format!("Invalid version_control_type: {}", e))?;

        Ok(Self {
            id: model.id,
            user_id: UserId(model.user_id),
            version_control_type,
            version_control_user_id: VersionControlUserId(model.version_control_user_id as i32),
            version_control_login: model.version_control_login,
            version_control_email: model.version_control_email,
            version_control_avatar_url: model.version_control_avatar_url,
            // TODO
            access_token: ReversibleCipherValue::new(model.access_token).unwrap(),
            refresh_token: model.refresh_token,
            token_type: model.token_type,
            expires_at: model.expires_at,
            scope: model.scope,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
