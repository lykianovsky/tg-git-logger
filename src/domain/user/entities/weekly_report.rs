use crate::domain::shared::date::range::DateRange;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VersionControlDateRangeReport {
    pub author: Option<String>,
    pub period: DateRange,
    pub commits: Vec<VersionControlDateRangeReportCommit>,
    pub pull_requests: Vec<VersionControlDateRangeReportPullRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionControlDateRangeReportCommit {
    pub sha: String,
    pub message: String,
    pub authored_at: DateTime<Utc>,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: Option<i64>,
    pub author: Option<VersionControlDateRangeReportAuthor>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionControlDateRangeReportPullRequest {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionControlDateRangeReportAuthor {
    pub login: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
}
