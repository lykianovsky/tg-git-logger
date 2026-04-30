use crate::application::webhook::commands::dispatch_event::command::DispatchWebhookEventExecutorCommand;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::domain::shared::command::CommandExecutor;
use crate::infrastructure::contracts::github::event_type::GithubEventType;
use crate::infrastructure::contracts::github::headers::GithubHeaders;
use crate::infrastructure::contracts::github::mapper::GithubWebhookEventMapper;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use reqwest::StatusCode;
use std::str::FromStr;
use std::sync::Arc;

pub struct AxumWebhookGithubController {}

impl AxumWebhookGithubController {
    pub async fn handle_post(
        headers: HeaderMap,
        Extension(executor): Extension<Arc<DispatchWebhookEventExecutor>>,
        Json(payload): Json<serde_json::Value>,
    ) -> StatusCode {
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

        tracing::debug!(event_type = %raw_event_type, "Received GitHub webhook event");

        let github_event_type = GithubEventType::from_str(raw_event_type)
            .unwrap_or(GithubEventType::Unknown(raw_event_type.to_string()));

        if github_event_type == GithubEventType::Ping {
            tracing::info!(event_type = %raw_event_type, "Received GitHub ping event");
            return StatusCode::OK;
        }

        let event = match GithubWebhookEventMapper::map_to_domain_event(&github_event_type, payload)
        {
            Ok(event) => event,
            Err(error) => {
                tracing::error!(error = ?error, event_type = %raw_event_type, "Failed to map GitHub event");
                return StatusCode::BAD_REQUEST;
            }
        };

        let cmd = DispatchWebhookEventExecutorCommand { event };

        match executor.execute(&cmd).await {
            Ok(_) => StatusCode::OK,
            Err(error) => {
                tracing::error!(error = ?error, "Failed to dispatch webhook event");
                StatusCode::BAD_REQUEST
            }
        }
    }
}
