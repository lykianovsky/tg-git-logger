use crate::server::webhook::github::events::GithubEventName;
use crate::server::webhook::github::pull_request::payload::PullRequestEvent;
use crate::server::webhook::github::push::payload::PushEvent;
use crate::server::webhook::github::release::payload::ReleaseEvent;
use crate::server::webhook::github::service::GithubWebhookService;
use crate::server::webhook::github::workflow::payload::WorkflowEvent;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use reqwest::StatusCode;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;

pub struct GithubHeaders;

impl GithubHeaders {
    pub const EVENT: &'static str = "x-github-event";
    // pub const DELIVERY: &'static str = "x-github-delivery";
    pub const SIGNATURE_256: &'static str = "x-hub-signature-256";
    // pub const HOOK_ID: &'static str = "x-github-hook-id";
    // pub const HOOK_INSTALLATION_TARGET_ID: &'static str = "x-github-hook-installation-target-id";
    // pub const HOOK_INSTALLATION_TARGET_TYPE: &'static str =
    //     "x-github-hook-installation-target-type";
}

pub struct GithubWebhookController {
    pub service: Arc<GithubWebhookService>,
}

impl GithubWebhookController {
    pub fn new(service: Arc<GithubWebhookService>) -> Self {
        Self { service }
    }

    pub fn handle(&self, headers: HeaderMap, body: Bytes) -> StatusCode {
        tracing::debug!(
            headers = ?headers,
            body_size = body.len(),
            "Incoming GitHub webhook request"
        );

        let raw_event_type = match headers
            .get(GithubHeaders::EVENT)
            .and_then(|header_value| header_value.to_str().ok())
        {
            Some(value) => value,
            None => {
                tracing::error!(header = GithubHeaders::EVENT, "Missing GitHub event header");
                return StatusCode::FORBIDDEN;
            }
        };

        tracing::info!(event = raw_event_type, "GitHub webhook raw event received");

        let event_name = GithubEventName::from_str(raw_event_type)
            .unwrap_or(GithubEventName::Unknown(raw_event_type.to_string()));

        tracing::debug!(
            parsed_event = ?event_name,
            "Parsed GitHub event name"
        );

        let payload: Value = match serde_json::from_slice(&body) {
            Ok(value) => {
                tracing::trace!("Payload JSON parsed successfully");
                value
            }
            Err(error) => {
                tracing::error!(
                    error = %error,
                    "Failed to parse webhook payload JSON"
                );
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        let status = match event_name {
            GithubEventName::Push => self.service.handle::<PushEvent>(payload),
            GithubEventName::PullRequest => self.service.handle::<PullRequestEvent>(payload),
            GithubEventName::Release => self.service.handle::<ReleaseEvent>(payload),
            GithubEventName::Workflow => self.service.handle::<WorkflowEvent>(payload),
            GithubEventName::Issues => StatusCode::NO_CONTENT,
            GithubEventName::Ping => {
                tracing::debug!("Ping event received");
                StatusCode::NO_CONTENT
            }
            GithubEventName::Unknown(name) => {
                tracing::warn!(
                    event = %name,
                    "Unknown GitHub event type"
                );
                StatusCode::FORBIDDEN
            }
        };

        tracing::debug!(
            status = %status,
            "Webhook handled"
        );

        status
    }
}

pub async fn handle(
    State(controller): State<Arc<GithubWebhookController>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    controller.handle(headers, body)
}
