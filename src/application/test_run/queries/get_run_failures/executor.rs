use crate::application::test_run::queries::get_run_failures::error::GetRunFailuresError;
use crate::application::test_run::queries::get_run_failures::query::GetRunFailuresQuery;
use crate::application::test_run::queries::get_run_failures::response::{
    GetRunFailuresResponse, TestFailureWithCard,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_failure_card_repository::TestFailureCardRepository;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use std::sync::Arc;

pub struct GetRunFailuresExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    card_repo: Arc<dyn TestFailureCardRepository>,
}

impl GetRunFailuresExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        card_repo: Arc<dyn TestFailureCardRepository>,
    ) -> Self {
        Self {
            test_run_repo,
            card_repo,
        }
    }
}

impl CommandExecutor for GetRunFailuresExecutor {
    type Command = GetRunFailuresQuery;
    type Response = GetRunFailuresResponse;
    type Error = GetRunFailuresError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        let run = self.test_run_repo.find_by_id(query.test_run_id).await?;
        let failures = self.test_run_repo.list_failures(query.test_run_id).await?;

        let mut result = Vec::with_capacity(failures.len());

        for failure in failures {
            let card = self
                .card_repo
                .find_by_fingerprint(run.repository_id, &failure.fingerprint)
                .await?;

            result.push(TestFailureWithCard { failure, card });
        }

        Ok(GetRunFailuresResponse { failures: result })
    }
}
