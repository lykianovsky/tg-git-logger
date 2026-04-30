use crate::domain::webhook::events::pull_request_review::{
    WebhookPullRequestReviewEvent, WebhookPullRequestReviewState,
};
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubPullRequestReviewEvent {
    pub review: GithubReview,
    pub pull_request: GithubReviewPullRequest,
    pub repository: GithubReviewRepository,
}

#[derive(Debug, Deserialize)]
pub struct GithubReview {
    pub id: u64,
    pub user: GithubReviewUser,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubReviewPullRequest {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub user: GithubReviewUser,
    #[serde(default)]
    pub review_comments: u64,
    #[serde(default)]
    pub mergeable_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubReviewRepository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubReviewUser {
    pub login: String,
}

impl GithubEvent for GithubPullRequestReviewEvent {
    type WebhookEvent = WebhookPullRequestReviewEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        let state = match self.review.state.as_str() {
            "approved" => WebhookPullRequestReviewState::Approved,
            "changes_requested" => WebhookPullRequestReviewState::ChangesRequested,
            "commented" => WebhookPullRequestReviewState::Commented,
            _ => WebhookPullRequestReviewState::Unknown,
        };

        WebhookPullRequestReviewEvent {
            reviewer: self.review.user.login.clone(),
            pr_author: self.pull_request.user.login.clone(),
            repo: self.repository.full_name.clone(),
            pr_number: self.pull_request.number,
            pr_title: self.pull_request.title.clone(),
            pr_url: self.pull_request.html_url.clone(),
            review_url: self.review.html_url.clone(),
            review_body: self.review.body.clone().filter(|b| !b.trim().is_empty()),
            state,
            review_comments: self.pull_request.review_comments,
            mergeable_state: self.pull_request.mergeable_state.clone(),
        }
    }
}
