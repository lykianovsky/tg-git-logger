use crate::application::user::commands::toggle_user_active::command::ToggleUserActiveCommand;
use crate::application::user::commands::toggle_user_active::error::ToggleUserActiveExecutorError;
use crate::application::user::commands::toggle_user_active::response::ToggleUserActiveResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_repository::UserRepository;
use std::sync::Arc;

pub struct ToggleUserActiveExecutor {
    user_repo: Arc<dyn UserRepository>,
}

impl ToggleUserActiveExecutor {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

impl CommandExecutor for ToggleUserActiveExecutor {
    type Command = ToggleUserActiveCommand;
    type Response = ToggleUserActiveResponse;
    type Error = ToggleUserActiveExecutorError;

    async fn execute(
        &self,
        cmd: &ToggleUserActiveCommand,
    ) -> Result<ToggleUserActiveResponse, ToggleUserActiveExecutorError> {
        self.user_repo
            .set_active(cmd.user_id, cmd.is_active)
            .await
            .map_err(|e| match e {
                crate::domain::user::repositories::user_repository::SetUserActiveError::DbError(msg) => {
                    ToggleUserActiveExecutorError::DbError(msg)
                }
                crate::domain::user::repositories::user_repository::SetUserActiveError::NotFound => {
                    ToggleUserActiveExecutorError::NotFound
                }
            })?;

        Ok(ToggleUserActiveResponse)
    }
}
