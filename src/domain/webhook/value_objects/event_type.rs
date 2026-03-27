use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum WebhookEventType {
    Ping,
    Push,
    PullRequest,
    PullRequestReview,
    PullRequestReviewComment,
    IssueComment,
    Issues,
    Release,
    Workflow,
    Unknown(String),
}

impl FromStr for WebhookEventType {
    type Err = ();

    fn from_str(external_string: &str) -> Result<Self, Self::Err> {
        match external_string {
            "push" => Ok(WebhookEventType::Push),
            "ping" => Ok(WebhookEventType::Ping),
            "pull_request" => Ok(WebhookEventType::PullRequest),
            "pull_request_review" => Ok(WebhookEventType::PullRequestReview),
            "pull_request_review_comment" => Ok(WebhookEventType::PullRequestReviewComment),
            "issue_comment" => Ok(WebhookEventType::IssueComment),
            "issues" => Ok(WebhookEventType::Issues),
            "release" => Ok(WebhookEventType::Release),
            "workflow_run" => Ok(WebhookEventType::Workflow),
            other => Ok(WebhookEventType::Unknown(other.to_string())),
        }
    }
}
