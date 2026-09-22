use crate::application::test_run::queries::get_last_test_run::error::GetLastTestRunError;
use crate::application::test_run::queries::get_last_test_run::query::GetLastTestRunQuery;
use crate::application::test_run::queries::get_last_test_run::response::GetLastTestRunResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use std::sync::Arc;

pub struct GetLastTestRunExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
}

impl GetLastTestRunExecutor {
    pub fn new(test_run_repo: Arc<dyn TestRunRepository>) -> Self {
        Self { test_run_repo }
    }
}

impl CommandExecutor for GetLastTestRunExecutor {
    type Command = GetLastTestRunQuery;
    type Response = GetLastTestRunResponse;
    type Error = GetLastTestRunError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self.test_run_repo.find_last(query.repository_id).await?;

        Ok(GetLastTestRunResponse { run })
    }
}
