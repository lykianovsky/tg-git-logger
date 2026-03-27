use crate::application::user::commands::bind_repository::command::BindRepositoryCommand;
use crate::application::user::commands::bind_repository::error::BindRepositoryExecutorError;
use crate::application::user::commands::bind_repository::response::BindRepositoryResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::entities::user_connection_repository::UserConnectionRepository;
use crate::domain::user::repositories::user_connection_repositories_repository::{
    CreateUserConnectionRepositoryError, UserConnectionRepositoriesRepository,
};
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use chrono::Utc;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct BindRepositoryExecutor {
    db: Arc<DatabaseConnection>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
    user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
}

impl BindRepositoryExecutor {
    pub fn new(
        db: Arc<DatabaseConnection>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
        user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    ) -> Self {
        Self {
            db,
            user_socials_repo,
            repository_repo,
            user_connection_repositories_repo,
        }
    }
}

impl CommandExecutor for BindRepositoryExecutor {
    type Command = BindRepositoryCommand;
    type Response = BindRepositoryResponse;
    type Error = BindRepositoryExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_user = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        self.repository_repo.find_by_id(cmd.repository_id).await?;

        let existing = self
            .user_connection_repositories_repo
            .find_by_user_id_and_repository_id(social_user.user_id, cmd.repository_id)
            .await?;

        if existing.is_some() {
            return Err(BindRepositoryExecutorError::AlreadyBound);
        }

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| BindRepositoryExecutorError::DbError(e.to_string()))?;

        let connection = UserConnectionRepository {
            id: Default::default(),
            user_id: social_user.user_id,
            repository_id: cmd.repository_id,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        match self
            .user_connection_repositories_repo
            .create(&txn, &connection)
            .await
        {
            Ok(_) => {}
            Err(CreateUserConnectionRepositoryError::DuplicateEntry) => {
                return Err(BindRepositoryExecutorError::AlreadyBound);
            }
            Err(e) => return Err(BindRepositoryExecutorError::CreateError(e)),
        }

        txn.commit()
            .await
            .map_err(|e| BindRepositoryExecutorError::DbError(e.to_string()))?;

        Ok(BindRepositoryResponse)
    }
}
