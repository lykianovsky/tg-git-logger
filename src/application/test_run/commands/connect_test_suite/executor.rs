use crate::application::test_run::commands::connect_test_suite::command::ConnectTestSuiteCommand;
use crate::application::test_run::commands::connect_test_suite::error::ConnectTestSuiteError;
use crate::application::test_run::commands::connect_test_suite::response::ConnectTestSuiteResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::entities::test_suite::NewTestSuite;
use crate::domain::test_run::repositories::test_suite_repository::TestSuiteRepository;
use std::sync::Arc;

pub struct ConnectTestSuiteExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
}

impl ConnectTestSuiteExecutor {
    pub fn new(test_suite_repo: Arc<dyn TestSuiteRepository>) -> Self {
        Self { test_suite_repo }
    }
}

impl CommandExecutor for ConnectTestSuiteExecutor {
    type Command = ConnectTestSuiteCommand;
    type Response = ConnectTestSuiteResponse;
    type Error = ConnectTestSuiteError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let workflow_file = cmd.workflow_file.trim();
        let default_ref = cmd.default_ref.trim();

        if workflow_file.is_empty() {
            return Err(ConnectTestSuiteError::EmptyWorkflowFile);
        }

        if default_ref.is_empty() {
            return Err(ConnectTestSuiteError::EmptyDefaultRef);
        }

        self.test_suite_repo
            .upsert(&NewTestSuite {
                repository_id: cmd.repository_id,
                workflow_file: workflow_file.to_string(),
                default_ref: default_ref.to_string(),
            })
            .await?;

        tracing::info!(
            repository_id = %cmd.repository_id.0,
            workflow_file = %workflow_file,
            default_ref = %default_ref,
            "Test suite connected"
        );

        Ok(ConnectTestSuiteResponse {})
    }
}
