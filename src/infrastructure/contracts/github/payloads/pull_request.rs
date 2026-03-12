use crate::domain::webhook::events::pull_request::WebhookPullRequestEvent;
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubPullRequestEvent {
    pub action: String, // opened, closed, reopened, synchronize и т.д.
    pub number: u64,
    pub pull_request: GithubPullRequest,
    pub repository: GithubRepository,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubPullRequest {
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,

    pub state: String,
    pub draft: bool,

    pub user: GithubUser,
    pub assignee: Option<GithubUser>,
    pub assignees: Vec<GithubUser>,

    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,

    pub merge_commit_sha: Option<String>,
    pub merged: bool,
    pub merged_by: Option<GithubUser>,

    pub commits: u64,
    pub additions: u64,
    pub deletions: u64,
    pub changed_files: u64,

    pub base: GithubPullRequestBranch,
    pub head: GithubPullRequestBranch,
}

#[derive(Debug, Deserialize)]
pub struct GithubPullRequestBranch {
    pub label: String, // user:branch
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    pub repo: GithubRepository,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub avatar_url: Option<String>,
    pub id: u64,
}

impl GithubEvent for GithubPullRequestEvent {
    type WebhookEvent = WebhookPullRequestEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        let pr = &self.pull_request;

        WebhookPullRequestEvent {
            source: self.sender.login.clone(),
            author: pr.user.login.clone(),
            repo: self.repository.full_name.clone(),
            repo_url: self.repository.html_url.clone(),
            title: pr.title.clone(),
            number: self.number,
            action: self.action.clone(),
            merged: pr.merged,
            merged_by: pr.merged_by.as_ref().map(|u| u.login.clone()),
            draft: pr.draft,
            state: pr.state.clone(),
            head_ref: pr.head.ref_field.clone(),
            base_ref: pr.base.ref_field.to_string(),
            head_repo: pr.head.label.clone(), // уже в формате owner:branch
            base_repo: pr.base.label.clone(),
            pr_url: Some(pr.html_url.clone()),
            merge_commit: pr.merge_commit_sha.clone(),
            assignees: pr.assignees.iter().map(|u| u.login.clone()).collect(),
            created_at: format_datetime(&pr.created_at),
            updated_at: format_datetime(&pr.updated_at),
            merged_at: pr.merged_at.as_deref().map(format_datetime),
            commits: pr.commits,
            additions: pr.additions,
            deletions: pr.deletions,
            changed_files: pr.changed_files,
        }
    }
}

fn format_datetime(ts: &str) -> String {
    DateTime::parse_from_rfc3339(ts)
        .map(|dt| {
            dt.with_timezone(&Local)
                .format("%d.%m.%Y %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|_| ts.to_string())
}
