use crate::domain::user::entities::User;
use crate::domain::user::repository::{
    BindGitHubException, FindUserByTgIdException, UserRepository,
};
use crate::domain::user::value_objects::{UserGithubAccount, UserTelegramId};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindGithubError {
    #[error("User not found: {0}")]
    UserNotFound(#[from] FindUserByTgIdException),

    #[error("GitHub account '{0}' is already bound to another user")]
    GithubAlreadyBound(String),

    #[error("Failed to bind GitHub account: {0}")]
    BindException(#[from] BindGitHubException),

    #[error("Database error: {0}")]
    DbError(String),
}

pub struct BindGithubUseCase {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
}

impl BindGithubUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { user_repo }
    }

    /// Привязать GitHub аккаунт к существующему пользователю
    pub async fn execute(
        &self,
        telegram_id: UserTelegramId,
        github_account: UserGithubAccount,
    ) -> Result<User, BindGithubError> {
        // 2️⃣ Проверяем, не привязан ли уже GitHub аккаунт с таким ID
        if let Ok(_) = self
            .user_repo
            .find_by_github_login(github_account.login.clone())
            .await
        {
            return Err(BindGithubError::GithubAlreadyBound(github_account.login));
        }

        let user = self
            .user_repo
            .find_by_tg_id(telegram_id.clone())
            .await
            .map_err(BindGithubError::UserNotFound)?;

        let mut new_user = user.clone();

        new_user.github = Some(github_account.clone());

        if let Err(e) = self
            .user_repo
            .bind_github(user.id.clone(), github_account)
            .await
        {
            return Err(BindGithubError::BindException(e));
        }

        Ok(new_user)
    }
}
