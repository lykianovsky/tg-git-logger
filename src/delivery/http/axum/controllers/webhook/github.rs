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

        let github_event_type = GithubEventType::from_str(raw_event_type)
            .unwrap_or(GithubEventType::Unknown(raw_event_type.to_string()));

        tracing::debug!("Parsed GitHub event type: {:?}", github_event_type);

        if github_event_type == GithubEventType::Ping {
            tracing::info!("Received ping-pong GitHub event type: {}", raw_event_type);
            return StatusCode::OK;
        }

        let event = match GithubWebhookEventMapper::map_to_domain_event(&github_event_type, payload)
        {
            Ok(event) => event,
            Err(error) => {
                tracing::error!("Failed to map GitHub event: {:?}", error);
                return StatusCode::BAD_REQUEST;
            }
        };

        let cmd = DispatchWebhookEventExecutorCommand { event };

        match executor.execute(&cmd).await {
            Ok(result) => {
                tracing::debug!("{:?}", result);
                StatusCode::OK
            }
            Err(error) => {
                tracing::error!("{:?}", error);
                StatusCode::BAD_REQUEST
            }
        }
    }
}
