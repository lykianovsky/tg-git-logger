use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug)]
pub struct VersionControlClientUser {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
}

#[derive(Debug, Error)]
pub enum VersionControlClientError {
    #[error("Network error: {0}")]
    Transport(String),
    #[error("User not found")]
    NotFound,
    #[error("Invalid access token")]
    Unauthorized,
    #[error("{0}")]
    Other(String),
}

#[async_trait]
pub trait VersionControlClient: Send + Sync {
    async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VersionControlClientUser, VersionControlClientError>;
}
