use crate::domain::user::entities::user::User;
use crate::domain::user::repositories::user_repository::{
    CreateUserError, FindUserByIdError, UserRepository,
};
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::users;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::sync::Arc;

pub struct MySQLUserRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for MySQLUserRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        user: &User,
    ) -> Result<User, CreateUserError> {
        let user_model = users::ActiveModel {
            is_active: Set(user.is_active as i8),
            ..Default::default()
        };

        let user = user_model
            .insert(txn)
            .await
            .map_err(|e| CreateUserError::DbError(e.to_string()))?;

        Ok(User::from_mysql(user))
    }

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdError> {
        let user = users::Entity::find()
            .filter(users::Column::Id.eq(id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByIdError::DbError(e.to_string()))?
            .ok_or(FindUserByIdError::NotFound)?;

        Ok(User::from_mysql(user))
    }
}

impl User {
    pub fn from_mysql(mysql_user: users::Model) -> Self {
        Self {
            id: UserId(mysql_user.id),
            is_active: mysql_user.is_active != 0,
            create_at: mysql_user.created_at,
            update_at: mysql_user.updated_at,
        }
    }
}
