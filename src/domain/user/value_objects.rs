use std::fmt;

#[derive(Debug, Clone)]
pub struct UserTelegramAccount {
    pub telegram_id: UserTelegramId,
    pub username: Option<String>,
    pub chat_id: i64,
}

#[derive(Debug, Clone)]
pub struct UserGithubAccount {
    pub github_id: UserGithubId,
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub(crate) fn to_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "{}", self.to_str().to_string()),
            UserRole::User => write!(f, "{}", self.to_str().to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UserTelegramId(pub i32);

#[derive(Debug, Clone, Copy)]
pub struct UserGithubId(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct UserId(pub i32);
