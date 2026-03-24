use crate::domain::shared::date::range::DateRange;
use crate::domain::version_control::value_objects::report::VersionControlDateRangeReport;
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

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[derive(Debug, Error)]
pub enum VersionControlClientDateRangeReportError {
    #[error("Network error: {0}")]
    Transport(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[async_trait]
pub trait VersionControlClient: Send + Sync {
    async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VersionControlClientGetUserResponse, VersionControlClientGetUserError>;

    async fn get_details_by_range(
        &self,
        access_token: &str,
        branch: String,
        range: &DateRange,
        author: Option<&str>,
    ) -> Result<VersionControlDateRangeReport, VersionControlClientDateRangeReportError>;
}
