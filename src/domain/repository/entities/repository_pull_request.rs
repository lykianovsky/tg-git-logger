use crate::domain::repository::value_objects::pull_request_status::PullRequestStatus;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPullRequest {
    pub id: i32,
    pub repository_id: RepositoryId,
    pub pr_number: i32,
    pub title: String,
    pub author: String,
    pub status: PullRequestStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
