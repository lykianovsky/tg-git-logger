use crate::domain::webhook::events::workflow::WebhookWorkflowEvent;
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowEvent {
    pub action: String, // "queued", "in_progress", "completed", "requested", etc.
    pub workflow_job: Option<GithubWorkflowJob>,
    pub workflow_run: Option<GithubWorkflowRun>,
    pub repository: GithubRepository,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowJob {
    pub id: u64,
    pub run_id: u64,
    pub run_url: String,
    pub html_url: String,
    pub status: String,             // "queued", "in_progress", "completed"
    pub conclusion: Option<String>, // "success", "failure", "cancelled" etc.
    pub name: String,
    pub steps: Option<Vec<GithubWorkflowStep>>,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: GithubCommitAuthor,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowRun {
    pub id: u64,
    pub name: String,
    pub html_url: String,
    pub status: String, // "queued", "in_progress", "completed"
    pub conclusion: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub head_commit: Option<GithubCommitInfo>,
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

impl GithubEvent for GithubWorkflowEvent {
    type WebhookEvent = WebhookWorkflowEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        Ok(serde_json::from_value(value)?)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        let run = self.workflow_run.as_ref();

        WebhookWorkflowEvent {
            id: run.map(|r| r.id).unwrap_or(0),
            name: run.map(|r| r.name.clone()).unwrap_or_default(),
            run_number: 0,              // GitHub webhook не присылает, оставляем 0
            head_branch: String::new(), // не приходит напрямую
            head_sha: run
                .and_then(|r| r.head_commit.as_ref())
                .map(|c| c.id.clone())
                .unwrap_or_default(),
            status: run
                .map(|r| r.status.clone())
                .unwrap_or_else(|| self.action.clone()),
            conclusion: run.and_then(|r| r.conclusion.clone()),
            html_url: run.map(|r| r.html_url.clone()),
            actor: Some(self.sender.login.clone()),
            repo: self.repository.full_name.clone(),
            repo_url: self.repository.html_url.clone(),
            created_at: run.map(|r| r.created_at.clone()),
            updated_at: run.map(|r| r.updated_at.clone()),
        }
    }
}
