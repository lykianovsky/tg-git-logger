use crate::domain::user::entities::User;
use crate::domain::user::value_objects::{UserGithubId, UserId, UserTelegramId};

#[derive(Debug)]
pub enum CreateUserException {
    DbError(String),
}


#[derive(Debug)]
pub enum FindUserByIdException {
    DbError(String),
}


#[derive(Debug)]
pub enum FindUserByTgIdException {
    DbError(String),
}


#[derive(Debug)]
pub enum FindUserByGitHubIdException {
    DbError(String),
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), CreateUserException>;

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdException>;

    async fn find_by_tg_id(&self, id: UserTelegramId) -> Result<User, FindUserByTgIdException>;

    async fn find_by_github_id(&self, id: UserGithubId) -> Result<User, FindUserByGitHubIdException>;
}