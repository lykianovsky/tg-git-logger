use crate::domain::webhook::events::pr_comment::WebhookPrCommentEvent;
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use serde::Deserialize;
use serde_json::Value;

/// Payload for `pull_request_review_comment` events.
#[derive(Debug, Deserialize)]
pub struct GithubPrReviewCommentEvent {
    pub action: String,
    pub pull_request: GithubCommentPr,
    pub comment: GithubComment,
    pub repository: GithubCommentRepository,
}

/// Payload for `issue_comment` events on pull requests.
#[derive(Debug, Deserialize)]
pub struct GithubIssueCommentEvent {
    pub action: String,
    pub issue: GithubIssueWithPr,
    pub comment: GithubComment,
    pub repository: GithubCommentRepository,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommentPr {
    pub number: u64,
    pub title: String,
    pub user: GithubCommentUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubIssueWithPr {
    pub number: u64,
    pub title: String,
    pub user: GithubCommentUser,
    /// Only present when the issue is actually a pull request.
    pub pull_request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GithubComment {
    pub body: String,
    pub html_url: String,
    pub user: GithubCommentUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommentUser {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommentRepository {
    pub full_name: String,
}

// ── GithubEvent impls ─────────────────────────────────────────────────────────

impl GithubEvent for GithubPrReviewCommentEvent {
    type WebhookEvent = WebhookPrCommentEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        WebhookPrCommentEvent {
            commenter: self.comment.user.login.clone(),
            pr_author: self.pull_request.user.login.clone(),
            repo: self.repository.full_name.clone(),
            pr_number: self.pull_request.number,
            pr_title: self.pull_request.title.clone(),
            comment_body: self.comment.body.clone(),
            comment_url: self.comment.html_url.clone(),
        }
    }
}

impl GithubEvent for GithubIssueCommentEvent {
    type WebhookEvent = WebhookPrCommentEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        WebhookPrCommentEvent {
            commenter: self.comment.user.login.clone(),
            pr_author: self.issue.user.login.clone(),
            repo: self.repository.full_name.clone(),
            pr_number: self.issue.number,
            pr_title: self.issue.title.clone(),
            comment_body: self.comment.body.clone(),
            comment_url: self.comment.html_url.clone(),
        }
    }
}
