use async_trait::async_trait;

#[derive(Debug)]
pub struct VersionControlClientUser {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
}

#[derive(Debug)]
pub enum VersionControlClientError {
    Transport(String),
    NotFound,
    Unauthorized,
    Other(String),
}

#[async_trait]
pub trait VersionControlClient: Send + Sync {
    async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VersionControlClientUser, VersionControlClientError>;
}
