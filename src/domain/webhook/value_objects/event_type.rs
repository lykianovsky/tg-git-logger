use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum WebhookEventType {
    Ping,
    Push,
    PullRequest,
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
            "issues" => Ok(WebhookEventType::Issues),
            "release" => Ok(WebhookEventType::Release),
            "workflow_run" => Ok(WebhookEventType::Workflow),
            other => Ok(WebhookEventType::Unknown(other.to_string())),
        }
    }
}
