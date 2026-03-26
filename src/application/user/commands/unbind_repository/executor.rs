use crate::application::user::commands::unbind_repository::command::UnbindRepositoryCommand;
use crate::application::user::commands::unbind_repository::error::UnbindRepositoryExecutorError;
use crate::application::user::commands::unbind_repository::response::UnbindRepositoryResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_connection_repositories_repository::UserConnectionRepositoriesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct UnbindRepositoryExecutor {
    db: Arc<DatabaseConnection>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
}

impl UnbindRepositoryExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    ) -> Self {
        Self {
            db,
            user_socials_repo,
            user_connection_repositories_repo,
        }
    }
}

impl CommandExecutor for UnbindRepositoryExecutor {
    type Command = UnbindRepositoryCommand;
    type Response = UnbindRepositoryResponse;
    type Error = UnbindRepositoryExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_user = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| UnbindRepositoryExecutorError::DbError(e.to_string()))?;

        self.user_connection_repositories_repo
            .delete_by_user_id_and_repository_id(&txn, social_user.user_id, cmd.repository_id)
            .await?;

        txn.commit()
            .await
            .map_err(|e| UnbindRepositoryExecutorError::DbError(e.to_string()))?;

        Ok(UnbindRepositoryResponse)
    }
}
