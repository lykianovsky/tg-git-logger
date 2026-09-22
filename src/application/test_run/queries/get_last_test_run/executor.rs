use crate::application::test_run::queries::get_last_test_run::error::GetLastTestRunError;
use crate::application::test_run::queries::get_last_test_run::query::GetLastTestRunQuery;
use crate::application::test_run::queries::get_last_test_run::response::GetLastTestRunResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use std::sync::Arc;

pub struct GetLastTestRunExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    test_suite_repo: Arc<dyn TestSuiteRepository>,
}

impl GetLastTestRunExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        test_suite_repo: Arc<dyn TestSuiteRepository>,
    ) -> Self {
        Self {
            test_run_repo,
            test_suite_repo,
        }
    }
}

impl CommandExecutor for GetLastTestRunExecutor {
    type Command = GetLastTestRunQuery;
    type Response = GetLastTestRunResponse;
    type Error = GetLastTestRunError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self.test_run_repo.find_last(query.repository_id).await?;
        let is_configured = self
            .test_suite_repo
            .find_by_repository(query.repository_id)
            .await
            .is_ok();

        Ok(GetLastTestRunResponse { run, is_configured })
    }
}
