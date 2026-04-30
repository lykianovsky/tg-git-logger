use crate::domain::shared::date::range::DateRange;
use crate::domain::version_control::value_objects::report::VersionControlDateRangeReport;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct OpenPullRequestSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub author_login: String,
    pub updated_at: DateTime<Utc>,
    pub requested_reviewers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UserPullRequestSummary {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub repo: String, // owner/repo
    pub author_login: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum VersionControlClientSearchPrsError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Transport error: {0}")]
    Transport(String),
}

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

    #[error("Branch not found: {0}")]
    BranchNotFound(String),
}

#[derive(Debug, Error)]
pub enum VersionControlClientBranchCheckError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Transport error: {0}")]
    Transport(String),
}

#[derive(Debug, Error)]
pub enum VersionControlClientPostCommentError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Transport error: {0}")]
    Transport(String),
}

#[derive(Debug, Error)]
pub enum VersionControlClientListPullRequestsError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Transport error: {0}")]
    Transport(String),
}

#[derive(Debug, Error)]
pub enum VersionControlClientGetPrError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found")]
    NotFound,

    #[error("Transport error: {0}")]
    Transport(String),
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
        owner: &str,
        repo: &str,
        branch: &str,
        range: &DateRange,
        author: Option<&str>,
    ) -> Result<VersionControlDateRangeReport, VersionControlClientDateRangeReportError>;

    async fn branch_exists(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<bool, VersionControlClientBranchCheckError>;

    async fn post_pr_comment(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> Result<(), VersionControlClientPostCommentError>;

    async fn list_open_pull_requests(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<OpenPullRequestSummary>, VersionControlClientListPullRequestsError>;

    async fn get_pr_mergeable_state(
        &self,
        access_token: &str,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Option<String>, VersionControlClientGetPrError>;

    async fn search_user_authored_open_prs(
        &self,
        access_token: &str,
        login: &str,
        repos: &[String],
    ) -> Result<Vec<UserPullRequestSummary>, VersionControlClientSearchPrsError>;

    async fn search_user_pending_reviews(
        &self,
        access_token: &str,
        login: &str,
        repos: &[String],
    ) -> Result<Vec<UserPullRequestSummary>, VersionControlClientSearchPrsError>;
}
