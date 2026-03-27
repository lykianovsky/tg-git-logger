use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::repositories::repository_repository::{
    CreateRepositoryError, DeleteRepositoryError, FindAllRepositoriesError
    , FindRepositoryByIdError, RepositoryRepository,
    UpdateRepositoryError,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::infrastructure::database::mysql::entities::repositories;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    ModelTrait, QueryFilter, Set,
};
use std::sync::Arc;

pub struct MySQLRepositoryRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLRepositoryRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RepositoryRepository for MySQLRepositoryRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        repository: &Repository,
    ) -> Result<Repository, CreateRepositoryError> {
        let model = repositories::ActiveModel {
            name: Set(repository.name.clone()),
            owner: Set(repository.owner.clone()),
            url: Set(repository.url.clone()),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateRepositoryError::DbError(e.to_string()))?;

        Ok(Repository::from_mysql(result))
    }

    async fn update(
        &self,
        txn: &DatabaseTransaction,
        repository: &Repository,
    ) -> Result<Repository, UpdateRepositoryError> {
        let model = repositories::ActiveModel {
            id: Set(repository.id.0),
            name: Set(repository.name.clone()),
            owner: Set(repository.owner.clone()),
            url: Set(repository.url.clone()),
            ..Default::default()
        };

        let result = model
            .update(txn)
            .await
            .map_err(|e| UpdateRepositoryError::DbError(e.to_string()))?;

        Ok(Repository::from_mysql(result))
    }

    async fn find_by_id(&self, id: RepositoryId) -> Result<Repository, FindRepositoryByIdError> {
        let result = repositories::Entity::find()
            .filter(repositories::Column::Id.eq(id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRepositoryByIdError::DbError(e.to_string()))?
            .ok_or(FindRepositoryByIdError::NotFound)?;

        Ok(Repository::from_mysql(result))
    }

    async fn find_all(&self) -> Result<Vec<Repository>, FindAllRepositoriesError> {
        let results = repositories::Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindAllRepositoriesError::DbError(e.to_string()))?;

        Ok(results.into_iter().map(Repository::from_mysql).collect())
    }

    async fn delete(
        &self,
        txn: &DatabaseTransaction,
        id: RepositoryId,
    ) -> Result<(), DeleteRepositoryError> {
        let model = repositories::Entity::find()
            .filter(repositories::Column::Id.eq(id.0))
            .one(txn)
            .await
            .map_err(|e| DeleteRepositoryError::DbError(e.to_string()))?
            .ok_or(DeleteRepositoryError::NotFound)?;

        model
            .delete(txn)
            .await
            .map_err(|e| DeleteRepositoryError::DbError(e.to_string()))?;

        Ok(())
    }
}

impl Repository {
    pub fn from_mysql(model: repositories::Model) -> Self {
        Self {
            id: RepositoryId(model.id),
            name: model.name,
            owner: model.owner,
            url: model.url,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
