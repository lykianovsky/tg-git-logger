use crate::applications::http::modules::webhook::github::events::pull_request::GithubWebhookPullRequestHandler;
use crate::applications::http::modules::webhook::github::events::push::GithubWebhookPushHandler;
use crate::applications::http::modules::webhook::github::events::release::GithubWebhookReleaseHandler;
use crate::applications::http::modules::webhook::github::events::workflow::GithubWebhookWorkflowHandler;
use crate::applications::http::modules::webhook::github::handler;
use crate::applications::state::ApplicationState;
use crate::infrastructure::contracts::github::events::GithubEventType;
use crate::infrastructure::contracts::github::headers::GithubHeaders;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::str::FromStr;
use std::sync::Arc;

pub struct GithubWebhookHandler {}

impl GithubWebhookHandler {
    pub async fn handle(
        State(state): State<Arc<ApplicationState>>,
        headers: HeaderMap,
        Json(payload): Json<serde_json::Value>,
    ) -> StatusCode {
        tracing::debug!("Received GitHub webhook payload: {:?}", payload);

        let raw_event_type = match headers
            .get(GithubHeaders::EVENT)
            .and_then(|header_value| header_value.to_str().ok())
        {
            Some(value) => value,
            None => {
                tracing::warn!(header = GithubHeaders::EVENT, "Missing GitHub event header");
                return StatusCode::FORBIDDEN;
            }
        };

        tracing::debug!("GitHub event header received: {}", raw_event_type);

        let event_type: GithubEventType = GithubEventType::from_str(raw_event_type)
            .unwrap_or(GithubEventType::Unknown(raw_event_type.to_string()));

        tracing::debug!("Parsed GitHub event type: {:?}", event_type);

        let status = match event_type {
            GithubEventType::Push => {
                tracing::debug!("Handling Push event");
                GithubWebhookPushHandler::new(state).handle(payload)
            },
            GithubEventType::PullRequest => {
                tracing::debug!("Handling PullRequest event");
                GithubWebhookPullRequestHandler::new(state).handle(payload)
            },
            GithubEventType::Release => {
                tracing::debug!("Handling Release event");
                GithubWebhookReleaseHandler::new(state).handle(payload)
            },
            GithubEventType::Workflow => {
                tracing::debug!("Handling Workflow event");
                GithubWebhookWorkflowHandler::new(state).handle(payload)
            },
            GithubEventType::Issues => {
                tracing::debug!("Issues event received - no action taken");
                StatusCode::NO_CONTENT
            },
            GithubEventType::Ping => {
                tracing::debug!("Ping event received - no action taken");
                StatusCode::NO_CONTENT
            }
            GithubEventType::Unknown(name) => {
                tracing::warn!(
                    event = %name,
                    "Unknown GitHub event type"
                );
                StatusCode::NO_CONTENT
            }
        };
        
        tracing::debug!("Returning status code: {}", status);
        status
    }
}
