use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_failure::{NewTestFailure, TestFailure};
use crate::domain::test_run::entities::test_run::{
    NewTestRun, TestRun, TestRunOutcome, TestRunTotals,
};
use crate::domain::test_run::repositories::test_run_repository::{
    CreateTestRunError, FindTestRunError, TestFailureCount, TestRunRepository, UpdateTestRunError,
};
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::{test_run_failures, test_runs};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::sync::Arc;

/// Прогоны старше этого срока планировщик считает зависшими и добирает их статус у CI
const STALE_RUN_MINUTES: i64 = 1;

pub struct MySQLTestRunRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLTestRunRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: test_runs::Model) -> Option<TestRun> {
        let status = TestRunStatus::from_str(&model.status)?;
        let trigger = TestRunTrigger::from_str(&model.trigger)?;

        // Итоги есть только у завершённого прогона, разобранного из отчёта
        let totals = model.total.map(|total| TestRunTotals {
            total: total.max(0) as u32,
            passed: model.passed.unwrap_or(0).max(0) as u32,
            failed: model.failed.unwrap_or(0).max(0) as u32,
            flaky: model.flaky.unwrap_or(0).max(0) as u32,
            skipped: model.skipped.unwrap_or(0).max(0) as u32,
            duration_ms: model.duration_ms.unwrap_or(0).max(0) as u64,
        });

        Some(TestRun {
            id: TestRunId(model.id),
            repository_id: RepositoryId(model.repository_id),
            run_tag: RunTag(model.run_tag),
            provider_run_id: model.provider_run_id.map(|id| id.max(0) as u64),
            run_url: model.run_url,
            git_ref: model.git_ref,
            sha: model.sha,
            trigger,
            args: model.args,
            requested_by_user_id: model.requested_by_user_id.map(UserId),
            chat_id: model.chat_id.map(SocialChatId),
            message_id: model.message_id,
            status,
            started_at: model.started_at,
            finished_at: model.finished_at,
            totals,
            created_at: model.created_at,
        })
    }

    fn failure_from_mysql(model: test_run_failures::Model) -> TestFailure {
        TestFailure {
            id: model.id,
            test_run_id: TestRunId(model.test_run_id),
            project: model.project,
            file: model.file,
            title: model.title,
            fingerprint: TestFingerprint(model.fingerprint),
            error_excerpt: model.error_excerpt,
        }
    }

    fn invalid_row_error() -> FindTestRunError {
        FindTestRunError::DbError("Invalid test run row".to_string())
    }
}

#[async_trait]
impl TestRunRepository for MySQLTestRunRepository {
    async fn create(&self, run: &NewTestRun) -> Result<TestRun, CreateTestRunError> {
        let model = test_runs::ActiveModel {
            repository_id: Set(run.repository_id.0),
            run_tag: Set(run.run_tag.0.clone()),
            git_ref: Set(run.git_ref.clone()),
            trigger: Set(run.trigger.as_str().to_string()),
            args: Set(run.args.clone()),
            requested_by_user_id: Set(run.requested_by_user_id.map(|id| id.0)),
            chat_id: Set(run.chat_id.map(|id| id.0)),
            status: Set(TestRunStatus::Queued.as_str().to_string()),
            ..Default::default()
        };

        let result = model
            .insert(self.db.as_ref())
            .await
            .map_err(|error| CreateTestRunError::DbError(error.to_string()))?;

        Self::from_mysql(result)
            .ok_or_else(|| CreateTestRunError::DbError("Invalid test run row".to_string()))
    }

    async fn find_by_id(&self, id: TestRunId) -> Result<TestRun, FindTestRunError> {
        let model = test_runs::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?
            .ok_or(FindTestRunError::NotFound)?;

        Self::from_mysql(model).ok_or_else(Self::invalid_row_error)
    }

    async fn find_by_tag(&self, tag: &RunTag) -> Result<TestRun, FindTestRunError> {
        let model = test_runs::Entity::find()
            .filter(test_runs::Column::RunTag.eq(tag.as_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?
            .ok_or(FindTestRunError::NotFound)?;

        Self::from_mysql(model).ok_or_else(Self::invalid_row_error)
    }

    async fn find_last(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<TestRun>, FindTestRunError> {
        let model = test_runs::Entity::find()
            .filter(test_runs::Column::RepositoryId.eq(repository_id.0))
            .order_by_desc(test_runs::Column::CreatedAt)
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        match model {
            Some(model) => Ok(Some(
                Self::from_mysql(model).ok_or_else(Self::invalid_row_error)?,
            )),
            None => Ok(None),
        }
    }

    async fn find_active(
        &self,
        repository_id: RepositoryId,
    ) -> Result<Option<TestRun>, FindTestRunError> {
        let active_statuses = [
            TestRunStatus::Queued.as_str(),
            TestRunStatus::Running.as_str(),
        ];

        let model = test_runs::Entity::find()
            .filter(test_runs::Column::RepositoryId.eq(repository_id.0))
            .filter(test_runs::Column::Status.is_in(active_statuses))
            .order_by_desc(test_runs::Column::CreatedAt)
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        match model {
            Some(model) => Ok(Some(
                Self::from_mysql(model).ok_or_else(Self::invalid_row_error)?,
            )),
            None => Ok(None),
        }
    }

    async fn list_recent(
        &self,
        repository_id: RepositoryId,
        limit: u64,
    ) -> Result<Vec<TestRun>, FindTestRunError> {
        let models = test_runs::Entity::find()
            .filter(test_runs::Column::RepositoryId.eq(repository_id.0))
            .order_by_desc(test_runs::Column::Id)
            .limit(limit)
            .all(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        Ok(models.into_iter().filter_map(Self::from_mysql).collect())
    }

    async fn count_failures_since(
        &self,
        repository_id: RepositoryId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<TestFailureCount>, FindTestRunError> {
        // Падения живут рядом с прогонами, поэтому сначала берём прогоны за период
        let run_ids: Vec<i32> = test_runs::Entity::find()
            .filter(test_runs::Column::RepositoryId.eq(repository_id.0))
            .filter(test_runs::Column::CreatedAt.gte(since))
            .all(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?
            .into_iter()
            .map(|run| run.id)
            .collect();

        if run_ids.is_empty() {
            return Ok(Vec::new());
        }

        let failures = test_run_failures::Entity::find()
            .filter(test_run_failures::Column::TestRunId.is_in(run_ids))
            .all(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        let mut counts: Vec<TestFailureCount> = Vec::new();

        for failure in failures {
            match counts
                .iter_mut()
                .find(|item| item.title == failure.title && item.file == failure.file)
            {
                Some(item) => item.count += 1,
                None => counts.push(TestFailureCount {
                    project: failure.project,
                    file: failure.file,
                    title: failure.title,
                    count: 1,
                }),
            }
        }

        counts.sort_by(|left, right| right.count.cmp(&left.count));

        Ok(counts)
    }

    async fn find_stale_active(&self, limit: u64) -> Result<Vec<TestRun>, FindTestRunError> {
        let active_statuses = [
            TestRunStatus::Queued.as_str(),
            TestRunStatus::Running.as_str(),
        ];
        let threshold = chrono::Utc::now() - chrono::Duration::minutes(STALE_RUN_MINUTES);

        let models = test_runs::Entity::find()
            .filter(test_runs::Column::Status.is_in(active_statuses))
            .filter(test_runs::Column::UpdatedAt.lt(threshold))
            .order_by_asc(test_runs::Column::UpdatedAt)
            .limit(limit)
            .all(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        Ok(models.into_iter().filter_map(Self::from_mysql).collect())
    }

    async fn mark_started(
        &self,
        id: TestRunId,
        provider_run_id: u64,
        run_url: String,
    ) -> Result<(), UpdateTestRunError> {
        let model = test_runs::ActiveModel {
            id: Set(id.0),
            provider_run_id: Set(Some(provider_run_id as i64)),
            run_url: Set(Some(run_url)),
            status: Set(TestRunStatus::Running.as_str().to_string()),
            started_at: Set(Some(chrono::Utc::now())),
            ..Default::default()
        };

        model
            .update(self.db.as_ref())
            .await
            .map_err(|error| UpdateTestRunError::DbError(error.to_string()))?;

        Ok(())
    }

    async fn save_outcome(
        &self,
        id: TestRunId,
        outcome: &TestRunOutcome,
    ) -> Result<(), UpdateTestRunError> {
        let mut model = test_runs::ActiveModel {
            id: Set(id.0),
            status: Set(outcome.status.as_str().to_string()),
            finished_at: Set(outcome.finished_at),
            ..Default::default()
        };

        if let Some(provider_run_id) = outcome.provider_run_id {
            model.provider_run_id = Set(Some(provider_run_id as i64));
        }

        if let Some(run_url) = &outcome.run_url {
            model.run_url = Set(Some(run_url.clone()));
        }

        if let Some(sha) = &outcome.sha {
            model.sha = Set(Some(sha.clone()));
        }

        if let Some(started_at) = outcome.started_at {
            model.started_at = Set(Some(started_at));
        }

        if let Some(totals) = outcome.totals {
            model.total = Set(Some(totals.total as i32));
            model.passed = Set(Some(totals.passed as i32));
            model.failed = Set(Some(totals.failed as i32));
            model.flaky = Set(Some(totals.flaky as i32));
            model.skipped = Set(Some(totals.skipped as i32));
            model.duration_ms = Set(Some(totals.duration_ms as i64));
        }

        model
            .update(self.db.as_ref())
            .await
            .map_err(|error| UpdateTestRunError::DbError(error.to_string()))?;

        Ok(())
    }

    async fn attach_message(
        &self,
        id: TestRunId,
        chat_id: SocialChatId,
        message_id: i32,
    ) -> Result<(), UpdateTestRunError> {
        let model = test_runs::ActiveModel {
            id: Set(id.0),
            chat_id: Set(Some(chat_id.0)),
            message_id: Set(Some(message_id)),
            ..Default::default()
        };

        model
            .update(self.db.as_ref())
            .await
            .map_err(|error| UpdateTestRunError::DbError(error.to_string()))?;

        Ok(())
    }

    async fn replace_failures(
        &self,
        id: TestRunId,
        failures: &[NewTestFailure],
    ) -> Result<(), UpdateTestRunError> {
        // Повторный вебхук того же прогона не должен задваивать список
        test_run_failures::Entity::delete_many()
            .filter(test_run_failures::Column::TestRunId.eq(id.0))
            .exec(self.db.as_ref())
            .await
            .map_err(|error| UpdateTestRunError::DbError(error.to_string()))?;

        if failures.is_empty() {
            return Ok(());
        }

        let models = failures
            .iter()
            .map(|failure| test_run_failures::ActiveModel {
                test_run_id: Set(id.0),
                project: Set(failure.project.clone()),
                file: Set(failure.file.clone()),
                title: Set(failure.title.clone()),
                fingerprint: Set(failure.fingerprint.0.clone()),
                error_excerpt: Set(failure.error_excerpt.clone()),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        test_run_failures::Entity::insert_many(models)
            .exec(self.db.as_ref())
            .await
            .map_err(|error| UpdateTestRunError::DbError(error.to_string()))?;

        Ok(())
    }

    async fn find_failure(&self, id: i32) -> Result<TestFailure, FindTestRunError> {
        let model = test_run_failures::Entity::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?
            .ok_or(FindTestRunError::NotFound)?;

        Ok(Self::failure_from_mysql(model))
    }

    async fn list_failures(&self, id: TestRunId) -> Result<Vec<TestFailure>, FindTestRunError> {
        let models = test_run_failures::Entity::find()
            .filter(test_run_failures::Column::TestRunId.eq(id.0))
            .order_by_asc(test_run_failures::Column::Id)
            .all(self.db.as_ref())
            .await
            .map_err(|error| FindTestRunError::DbError(error.to_string()))?;

        Ok(models.into_iter().map(Self::failure_from_mysql).collect())
    }
}
