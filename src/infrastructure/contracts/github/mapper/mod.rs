use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::contracts::github::event_type::{GithubEvent, GithubEventType};
use crate::infrastructure::contracts::github::payloads::pr_comment::{
    GithubIssueCommentEvent, GithubPrReviewCommentEvent,
};
use crate::infrastructure::contracts::github::payloads::pull_request::GithubPullRequestEvent;
use crate::infrastructure::contracts::github::payloads::pull_request_review::GithubPullRequestReviewEvent;
use crate::infrastructure::contracts::github::payloads::push::GithubPushEvent;
use crate::infrastructure::contracts::github::payloads::release::GithubReleaseEvent;
use crate::infrastructure::contracts::github::payloads::workflow::GithubWorkflowEvent;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubWebhookEventMapperError {
    #[error("Unsupported GitHub event type")]
    UnsupportedEventType,
    #[error("Invalid payload for GitHub event: {0}")]
    InvalidPayload(String),
}

pub struct GithubWebhookEventMapper;

impl GithubWebhookEventMapper {
    pub fn map_to_domain_event(
        github_event_type: &GithubEventType,
        payload: serde_json::Value,
    ) -> Result<Box<dyn WebhookEvent>, GithubWebhookEventMapperError> {
        match github_event_type {
            GithubEventType::Push => Self::parse_to_domain::<GithubPushEvent>(payload),
            GithubEventType::PullRequest => {
                Self::parse_to_domain::<GithubPullRequestEvent>(payload)
            }
            GithubEventType::PullRequestReview => {
                Self::parse_to_domain::<GithubPullRequestReviewEvent>(payload)
            }
            GithubEventType::Release => Self::parse_to_domain::<GithubReleaseEvent>(payload),
            GithubEventType::Workflow => Self::parse_to_domain::<GithubWorkflowEvent>(payload),
            GithubEventType::PullRequestReviewComment => {
                Self::parse_to_domain::<GithubPrReviewCommentEvent>(payload)
            }
            GithubEventType::IssueComment => {
                Self::parse_to_domain::<GithubIssueCommentEvent>(payload)
            }

            _ => {
                tracing::warn!(
                    "No mapping found for GitHub event type: {:?}",
                    github_event_type
                );
                Err(GithubWebhookEventMapperError::UnsupportedEventType)
            }
        }
    }

    fn parse_to_domain<E>(
        payload: serde_json::Value,
    ) -> Result<Box<dyn WebhookEvent>, GithubWebhookEventMapperError>
    where
        E: GithubEvent,
    {
        let event = E::from_value(payload)
            .map_err(|e| GithubWebhookEventMapperError::InvalidPayload(e.to_string()))?;

        Ok(Box::new(event.to_webhook_event()))
    }
}
