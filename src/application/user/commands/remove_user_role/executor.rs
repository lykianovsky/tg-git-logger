use crate::application::user::commands::remove_user_role::command::RemoveUserRoleCommand;
use crate::application::user::commands::remove_user_role::error::RemoveUserRoleExecutorError;
use crate::application::user::commands::remove_user_role::response::RemoveUserRoleResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use std::sync::Arc;

pub struct RemoveUserRoleExecutor {
    user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
}

impl RemoveUserRoleExecutor {
    pub fn new(user_has_roles_repo: Arc<dyn UserHasRolesRepository>) -> Self {
        Self {
            user_has_roles_repo,
        }
    }
}

impl CommandExecutor for RemoveUserRoleExecutor {
    type Command = RemoveUserRoleCommand;
    type Response = RemoveUserRoleResponse;
    type Error = RemoveUserRoleExecutorError;

    async fn execute(
        &self,
        cmd: &RemoveUserRoleCommand,
    ) -> Result<RemoveUserRoleResponse, RemoveUserRoleExecutorError> {
        self.user_has_roles_repo
            .remove(cmd.user_id, cmd.role_name.clone())
            .await
            .map_err(|e| RemoveUserRoleExecutorError::DbError(e.to_string()))?;

        Ok(RemoveUserRoleResponse)
    }
}
