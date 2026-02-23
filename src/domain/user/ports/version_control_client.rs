use crate::domain::shared::date::range::DateRange;
use crate::domain::user::entities::weekly_report::VersionControlDateRangeReport;
use crate::infrastructure::integrations::version_control::github::client::GithubClientError;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug)]
pub struct VersionControlClientGetUserResponse {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
}

#[derive(Debug, Error)]
pub enum VersionControlClientGetUserError {
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
    ) -> Result<VersionControlClientGetUserResponse, VersionControlClientGetUserError>;

    async fn get_report(
        &self,
        access_token: &str,
        range: &DateRange,
        author: Option<&str>,
    ) -> Result<VersionControlDateRangeReport, GithubClientError>;
}
