use crate::application::test_run::commands::attach_test_run_message::command::AttachTestRunMessageCommand;
use crate::application::test_run::commands::attach_test_run_message::error::AttachTestRunMessageError;
use crate::application::test_run::commands::attach_test_run_message::response::AttachTestRunMessageResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use std::sync::Arc;

pub struct AttachTestRunMessageExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
}

impl AttachTestRunMessageExecutor {
    pub fn new(test_run_repo: Arc<dyn TestRunRepository>) -> Self {
        Self { test_run_repo }
    }
}

impl CommandExecutor for AttachTestRunMessageExecutor {
    type Command = AttachTestRunMessageCommand;
    type Response = AttachTestRunMessageResponse;
    type Error = AttachTestRunMessageError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.test_run_repo
            .attach_message(cmd.test_run_id, cmd.chat_id, cmd.message_id)
            .await?;

        Ok(AttachTestRunMessageResponse {})
    }
}
