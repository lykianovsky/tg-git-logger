use crate::domain::user::entities::User;
use crate::domain::user::repository::{BindGitHubException, FindUserByTgIdException, UserRepository};
use crate::domain::user::value_objects::{UserGithubAccount, UserGithubId, UserTelegramId};
use std::sync::Arc;

#[derive(Debug)]
pub enum BindGithubError {
    UserNotFound(FindUserByTgIdException),
    GithubAlreadyBound(String),
    BindException(BindGitHubException),
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
        login: String,
    ) -> Result<User, BindGithubError> {
        // 2️⃣ Проверяем, не привязан ли уже GitHub аккаунт с таким ID
        if let Ok(_) = self.user_repo.find_by_github_login(login.clone()).await {
            return Err(BindGithubError::GithubAlreadyBound(format!(
                "GitHub login {} уже привязан к другому пользователю",
                login
            )));
        }

        let user = self
            .user_repo
            .find_by_tg_id(telegram_id.clone())
            .await
            .map_err(BindGithubError::UserNotFound)?;

        // 3️⃣ Создаем доменную сущность GitHub и привязываем
        let github_account = UserGithubAccount {
            github_id: UserGithubId(0),
            login: login.clone(),
        };

        let mut new_user = user.clone();

        new_user.github = Some(github_account.clone());

        if let Err(e) = self.user_repo.bind_github(user.id.clone(), github_account).await {
            return Err(BindGithubError::BindException(e));
        }

        Ok(new_user)
    }
}
