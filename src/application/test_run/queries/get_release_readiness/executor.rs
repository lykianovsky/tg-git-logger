use crate::application::test_run::queries::get_release_readiness::error::GetReleaseReadinessError;
use crate::application::test_run::queries::get_release_readiness::query::GetReleaseReadinessQuery;
use crate::application::test_run::queries::get_release_readiness::response::{
    GetReleaseReadinessResponse, ReleaseReadinessVerdict,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_run::TestRun;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use std::sync::Arc;

/// Зелёный прогон старше суток ничего не говорит о сегодняшнем состоянии
const FRESH_RUN_HOURS: i64 = 24;

pub struct GetReleaseReadinessExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
}

impl GetReleaseReadinessExecutor {
    pub fn new(test_run_repo: Arc<dyn TestRunRepository>) -> Self {
        Self { test_run_repo }
    }

    fn build_verdict(run: Option<&TestRun>) -> ReleaseReadinessVerdict {
        let Some(run) = run else {
            return ReleaseReadinessVerdict::NoData;
        };

        if run.is_active() {
            return ReleaseReadinessVerdict::Running;
        }

        let finished_at = run.finished_at.unwrap_or(run.created_at);
        let age = chrono::Utc::now() - finished_at;

        match run.status {
            TestRunStatus::Passed if age.num_hours() < FRESH_RUN_HOURS => {
                ReleaseReadinessVerdict::Ready
            }
            TestRunStatus::Passed => ReleaseReadinessVerdict::Stale,
            TestRunStatus::Failed => ReleaseReadinessVerdict::Failed,
            _ => ReleaseReadinessVerdict::NoData,
        }
    }
}

impl CommandExecutor for GetReleaseReadinessExecutor {
    type Command = GetReleaseReadinessQuery;
    type Response = GetReleaseReadinessResponse;
    type Error = GetReleaseReadinessError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self.test_run_repo.find_last(query.repository_id).await?;
        let verdict = Self::build_verdict(run.as_ref());

        // Список мешающих тестов нужен только когда релиз не готов из-за падений
        let failures = match (&run, verdict) {
            (Some(run), ReleaseReadinessVerdict::Failed) => {
                self.test_run_repo.list_failures(run.id).await?
            }
            _ => Vec::new(),
        };

        Ok(GetReleaseReadinessResponse {
            verdict,
            run,
            failures,
        })
    }
}
