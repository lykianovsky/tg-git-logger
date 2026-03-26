use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::entities::user_connection_repository::UserConnectionRepository;
use crate::domain::user::repositories::user_connection_repositories_repository::{
    CreateUserConnectionRepositoryError, FindUserConnectionRepositoriesByUserIdError,
    UserConnectionRepositoriesRepository,
};
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::user_connection_repositories;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::sync::Arc;

pub struct MySQLUserConnectionRepositoriesRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserConnectionRepositoriesRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserConnectionRepositoriesRepository for MySQLUserConnectionRepositoriesRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        connection: &UserConnectionRepository,
    ) -> Result<UserConnectionRepository, CreateUserConnectionRepositoryError> {
        let model = user_connection_repositories::ActiveModel {
            user_id: Set(connection.user_id.0),
            repository_id: Set(connection.repository_id.0),
            is_active: Set(connection.is_active as i8),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateUserConnectionRepositoryError::DbError(e.to_string()))?;

        Ok(UserConnectionRepository::from_mysql(result))
    }

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<UserConnectionRepository>, FindUserConnectionRepositoriesByUserIdError> {
        let results = user_connection_repositories::Entity::find()
            .filter(user_connection_repositories::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindUserConnectionRepositoriesByUserIdError::DbError(e.to_string()))?;

        Ok(results
            .into_iter()
            .map(UserConnectionRepository::from_mysql)
            .collect())
    }
}

impl UserConnectionRepository {
    pub fn from_mysql(model: user_connection_repositories::Model) -> Self {
        Self {
            id: model.id,
            user_id: UserId(model.user_id),
            repository_id: RepositoryId(model.repository_id),
            is_active: model.is_active != 0,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
