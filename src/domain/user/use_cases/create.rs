use crate::domain::user::repository::CreateUserException;
use crate::domain::user::value_objects::{UserGithubAccount, UserGithubId, UserId, UserRole, UserTelegramAccount, UserTelegramId};
use crate::domain::user::{entities::User, repository::UserRepository};

pub struct CreateUserUseCase<R: UserRepository> {
    pub repository: R,
}

impl<R: UserRepository> CreateUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<User, CreateUserException> {
        let user = User {
            id: Some(UserId(1)), // ID юзера
            telegram: Some(UserTelegramAccount {
                telegram_id: UserTelegramId(123),
                username: Some("alice_bot".into()),
                chat_id: 987654321,
            }),
            github: Some(UserGithubAccount {
                github_id: UserGithubId(123),
                login: "alice-github".into(),
            }),
            roles: vec![UserRole::Admin], // можно несколько ролей
        };

        self.repository.create(&user).await?;

        Ok(user)
    }
}
