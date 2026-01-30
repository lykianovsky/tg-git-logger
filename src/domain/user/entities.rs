use crate::domain::user::value_objects::{UserGithubAccount, UserId, UserRole, UserTelegramAccount};


#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub telegram: Option<UserTelegramAccount>,
    pub github: Option<UserGithubAccount>,
    pub roles: Vec<UserRole>,
}