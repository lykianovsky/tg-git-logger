use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_suite::{
    DEFAULT_ARGS_INPUT_NAME, DEFAULT_ARTIFACT_PREFIX, DEFAULT_REF_INPUT_NAME, DEFAULT_SUMMARY_PATH,
    DEFAULT_TAG_INPUT_NAME, DEFAULT_TESTS_ROOT, NewTestSuite, TestSuite,
};
use crate::domain::test_run::repositories::test_suite_repository::{
    FindTestSuiteError, SaveTestSuiteError, TestSuiteRepository,
};
use crate::infrastructure::database::mysql::entities::{repositories, test_suites};
use async_trait::async_trait;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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

    async fn upsert(&self, suite: &NewTestSuite) -> Result<(), SaveTestSuiteError> {
        let existing = test_suites::Entity::find()
            .filter(test_suites::Column::RepositoryId.eq(suite.repository_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|error| SaveTestSuiteError::DbError(error.to_string()))?;

        let now = chrono::Utc::now();

        match existing {
            Some(model) => {
                let mut active: test_suites::ActiveModel = model.into();

                active.workflow_file = Set(suite.workflow_file.clone());
                active.default_ref = Set(suite.default_ref.clone());
                active.updated_at = Set(now);

                active
                    .update(self.db.as_ref())
                    .await
                    .map_err(|error| SaveTestSuiteError::DbError(error.to_string()))?;
            }
            None => {
                test_suites::ActiveModel {
                    repository_id: Set(suite.repository_id.0),
                    workflow_file: Set(suite.workflow_file.clone()),
                    default_ref: Set(suite.default_ref.clone()),
                    args_input_name: Set(DEFAULT_ARGS_INPUT_NAME.to_string()),
                    ref_input_name: Set(DEFAULT_REF_INPUT_NAME.to_string()),
                    tag_input_name: Set(DEFAULT_TAG_INPUT_NAME.to_string()),
                    artifact_prefix: Set(DEFAULT_ARTIFACT_PREFIX.to_string()),
                    summary_path: Set(DEFAULT_SUMMARY_PATH.to_string()),
                    tests_root: Set(DEFAULT_TESTS_ROOT.to_string()),
                    created_at: Set(now),
                    updated_at: Set(now),
                    ..Default::default()
                }
                .insert(self.db.as_ref())
                .await
                .map_err(|error| SaveTestSuiteError::DbError(error.to_string()))?;
            }
        }

        Ok(())
    }
}
