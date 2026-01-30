use crate::domain::user::entities::User;
use crate::domain::user::value_objects::{UserGithubAccount, UserId, UserRole, UserTelegramId};

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
pub enum FindUserByGitHubLoginException {
    DbError(String),
}

#[derive(Debug)]
pub enum AssignUserRoleException {
    DbError(String),
}

#[derive(Debug)]
pub enum BindGitHubException {
    DbError(String),
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), CreateUserException>;

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdException>;

    async fn find_by_tg_id(&self, id: UserTelegramId) -> Result<User, FindUserByTgIdException>;

    async fn find_by_github_login(&self, id: String) -> Result<User, FindUserByGitHubLoginException>;

    async fn assign_role(&self, user_id: UserId, role: UserRole) -> Result<(), AssignUserRoleException>;

    async fn bind_github(&self, user_id: UserId, github: UserGithubAccount) -> Result<(), BindGitHubException>;
}