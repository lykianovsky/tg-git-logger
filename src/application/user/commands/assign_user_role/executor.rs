use crate::application::user::commands::assign_user_role::command::AssignUserRoleCommand;
use crate::application::user::commands::assign_user_role::error::AssignUserRoleExecutorError;
use crate::application::user::commands::assign_user_role::response::AssignUserRoleResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use sea_orm::DatabaseConnection;
use sea_orm::TransactionTrait;
use std::sync::Arc;

pub struct AssignUserRoleExecutor {
    db: Arc<DatabaseConnection>,
    user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
}

impl AssignUserRoleExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    ) -> Self {
        Self {
            db,
            user_has_roles_repo,
        }
    }
}

impl CommandExecutor for AssignUserRoleExecutor {
    type Command = AssignUserRoleCommand;
    type Response = AssignUserRoleResponse;
    type Error = AssignUserRoleExecutorError;

    async fn execute(
        &self,
        cmd: &AssignUserRoleCommand,
    ) -> Result<AssignUserRoleResponse, AssignUserRoleExecutorError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| AssignUserRoleExecutorError::DbError(e.to_string()))?;

        self.user_has_roles_repo
            .assign(&txn, cmd.user_id, cmd.role_name.clone())
            .await
            .map_err(|e| AssignUserRoleExecutorError::DbError(e.to_string()))?;

        txn.commit()
            .await
            .map_err(|e| AssignUserRoleExecutorError::DbError(e.to_string()))?;

        Ok(AssignUserRoleResponse)
    }
}
