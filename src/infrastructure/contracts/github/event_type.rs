use crate::domain::webhook::events::WebhookEvent;
use crate::domain::webhook::value_objects::event_type::WebhookEventType;
use std::str::FromStr;

pub trait GithubEvent {
    type WebhookEvent: WebhookEvent + Send + Sync + 'static;

    fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized;

    fn to_webhook_event(&self) -> Self::WebhookEvent;
}

#[derive(Debug, PartialEq)]
pub enum GithubEventType {
    Ping,
    Push,
    PullRequest,
    Issues,
    Release,
    Workflow,
    Unknown(String),
}

impl FromStr for GithubEventType {
    type Err = ();

    fn from_str(external_string: &str) -> Result<Self, Self::Err> {
        match external_string {
            "push" => Ok(GithubEventType::Push),
            "ping" => Ok(GithubEventType::Ping),
            "pull_request" => Ok(GithubEventType::PullRequest),
            "issues" => Ok(GithubEventType::Issues),
            "release" => Ok(GithubEventType::Release),
            "workflow_run" => Ok(GithubEventType::Workflow),
            other => Ok(GithubEventType::Unknown(other.to_string())),
        }
    }
}

impl From<GithubEventType> for WebhookEventType {
    fn from(event: GithubEventType) -> Self {
        match event {
            GithubEventType::Push => WebhookEventType::Push,
            GithubEventType::Ping => WebhookEventType::Ping,
            GithubEventType::PullRequest => WebhookEventType::PullRequest,
            GithubEventType::Issues => WebhookEventType::Issues,
            GithubEventType::Release => WebhookEventType::Release,
            GithubEventType::Workflow => WebhookEventType::Workflow,
            GithubEventType::Unknown(s) => WebhookEventType::Unknown(s),
        }
    }
}
