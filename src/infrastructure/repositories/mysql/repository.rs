use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::repositories::repository_repository::{
    CreateRepositoryError, FindAllRepositoriesError, FindRepositoryByExternalIdError,
    FindRepositoryByIdError, RepositoryRepository,
};
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::infrastructure::database::mysql::entities::repositories;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
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
            external_id: Set(repository.external_id),
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

    async fn find_by_id(&self, id: RepositoryId) -> Result<Repository, FindRepositoryByIdError> {
        let result = repositories::Entity::find()
            .filter(repositories::Column::Id.eq(id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRepositoryByIdError::DbError(e.to_string()))?
            .ok_or(FindRepositoryByIdError::NotFound)?;

        Ok(Repository::from_mysql(result))
    }

    async fn find_by_external_id(
        &self,
        external_id: i64,
    ) -> Result<Repository, FindRepositoryByExternalIdError> {
        let result = repositories::Entity::find()
            .filter(repositories::Column::ExternalId.eq(external_id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindRepositoryByExternalIdError::DbError(e.to_string()))?
            .ok_or(FindRepositoryByExternalIdError::NotFound)?;

        Ok(Repository::from_mysql(result))
    }

    async fn find_all(&self) -> Result<Vec<Repository>, FindAllRepositoriesError> {
        let results = repositories::Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindAllRepositoriesError::DbError(e.to_string()))?;

        Ok(results.into_iter().map(Repository::from_mysql).collect())
    }
}

impl Repository {
    pub fn from_mysql(model: repositories::Model) -> Self {
        Self {
            id: RepositoryId(model.id),
            external_id: model.external_id,
            name: model.name,
            owner: model.owner,
            url: model.url,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
