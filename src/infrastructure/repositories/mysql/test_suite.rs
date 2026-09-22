use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_suite::TestSuite;
use crate::domain::test_run::repositories::test_suite_repository::{
    FindTestSuiteError, TestSuiteRepository,
};
use crate::infrastructure::database::mysql::entities::{repositories, test_suites};
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

pub struct MySQLTestSuiteRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLTestSuiteRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: test_suites::Model, repository: repositories::Model) -> TestSuite {
        TestSuite {
            repository_id: RepositoryId(model.repository_id),
            owner: repository.owner,
            name: repository.name,
            workflow_file: model.workflow_file,
            default_ref: model.default_ref,
            args_input_name: model.args_input_name,
            ref_input_name: model.ref_input_name,
            tag_input_name: model.tag_input_name,
            artifact_prefix: model.artifact_prefix,
            summary_path: model.summary_path,
            tests_root: model.tests_root,
        }
    }
}

#[async_trait]
impl TestSuiteRepository for MySQLTestSuiteRepository {
    async fn find_by_repository(
        &self,
        repository_id: RepositoryId,
    ) -> Result<TestSuite, FindTestSuiteError> {
        let (model, repository) = test_suites::Entity::find()
            .filter(test_suites::Column::RepositoryId.eq(repository_id.0))
            .find_also_related(repositories::Entity)
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestSuiteError::DbError(error.to_string()))?
            .ok_or(FindTestSuiteError::NotConfigured)?;

        // Набор тестов без репозитория недостижим: связь обязательная с каскадом
        let repository = repository.ok_or(FindTestSuiteError::NotConfigured)?;

        Ok(Self::from_mysql(model, repository))
    }
}
