use crate::domain::repository::entities::repository_pull_request::RepositoryPullRequest;
use crate::domain::repository::repositories::repository_pull_request_repository::{
    CreatePullRequestError, FindPullRequestByIdError, FindPullRequestsByRepositoryIdError,
    RepositoryPullRequestRepository,
};
use crate::domain::repository::value_objects::pull_request_status::PullRequestStatus;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::infrastructure::database::mysql::entities::repository_pull_requests;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLRepositoryPullRequestRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLRepositoryPullRequestRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RepositoryPullRequestRepository for MySQLRepositoryPullRequestRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        pull_request: &RepositoryPullRequest,
    ) -> Result<RepositoryPullRequest, CreatePullRequestError> {
        let model = repository_pull_requests::ActiveModel {
            repository_id: Set(pull_request.repository_id.0),
            pr_number: Set(pull_request.pr_number),
            title: Set(pull_request.title.clone()),
            author: Set(pull_request.author.clone()),
            status: Set(pull_request.status.to_string()),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreatePullRequestError::DbError(e.to_string()))?;

        RepositoryPullRequest::from_mysql(result).map_err(CreatePullRequestError::DbError)
    }

    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<RepositoryPullRequest, FindPullRequestByIdError> {
        let result = repository_pull_requests::Entity::find()
            .filter(repository_pull_requests::Column::Id.eq(id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindPullRequestByIdError::DbError(e.to_string()))?
            .ok_or(FindPullRequestByIdError::NotFound)?;

        RepositoryPullRequest::from_mysql(result).map_err(FindPullRequestByIdError::DbError)
    }

    async fn find_by_repository_id(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Vec<RepositoryPullRequest>, FindPullRequestsByRepositoryIdError> {
        let results = repository_pull_requests::Entity::find()
            .filter(repository_pull_requests::Column::RepositoryId.eq(repository_id.0))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindPullRequestsByRepositoryIdError::DbError(e.to_string()))?;

        results
            .into_iter()
            .map(|m| {
                RepositoryPullRequest::from_mysql(m)
                    .map_err(FindPullRequestsByRepositoryIdError::DbError)
            })
            .collect()
    }
}

impl RepositoryPullRequest {
    pub fn from_mysql(model: repository_pull_requests::Model) -> Result<Self, String> {
        let status = PullRequestStatus::from_str(&model.status)
            .map_err(|e| format!("Invalid pull request status in DB: {}", e))?;

        Ok(Self {
            id: model.id,
            repository_id: RepositoryId(model.repository_id),
            pr_number: model.pr_number,
            title: model.title,
            author: model.author,
            status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
