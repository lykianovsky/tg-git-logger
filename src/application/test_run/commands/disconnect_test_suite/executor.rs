use crate::application::test_run::commands::disconnect_test_suite::command::DisconnectTestSuiteCommand;
use crate::application::test_run::commands::disconnect_test_suite::error::DisconnectTestSuiteError;
use crate::application::test_run::commands::disconnect_test_suite::response::DisconnectTestSuiteResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use std::sync::Arc;

pub struct DisconnectTestSuiteExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
}

impl DisconnectTestSuiteExecutor {
    pub fn new(test_suite_repo: Arc<dyn TestSuiteRepository>) -> Self {
        Self { test_suite_repo }
    }
}

impl CommandExecutor for DisconnectTestSuiteExecutor {
    type Command = DisconnectTestSuiteCommand;
    type Response = DisconnectTestSuiteResponse;
    type Error = DisconnectTestSuiteError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.test_suite_repo.delete(cmd.repository_id).await?;

        tracing::info!(
            repository_id = %cmd.repository_id.0,
            "Test suite disconnected"
        );

        Ok(DisconnectTestSuiteResponse {})
    }
}
