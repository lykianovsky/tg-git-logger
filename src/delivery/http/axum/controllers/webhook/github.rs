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
    #[tracing::instrument(
        name = "webhook.github",
        skip_all,
        fields(event_type = tracing::field::Empty)
    )]
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

        tracing::Span::current().record("event_type", raw_event_type);

        // Normalize event_type label to a small known set to avoid Prometheus
        // cardinality blow-up if random/unexpected event types arrive.
        let metric_label = match &github_event_type {
            GithubEventType::Unknown(_) => "unknown",
            _ => raw_event_type,
        };
        crate::infrastructure::metrics::registry::METRICS
            .webhook_received_total
            .with_label_values(&[metric_label])
            .inc();

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
                crate::infrastructure::metrics::registry::METRICS
                    .errors_total
                    .with_label_values(&["webhook", "map_failed"])
                    .inc();
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
                crate::infrastructure::metrics::registry::METRICS
                    .errors_total
                    .with_label_values(&["webhook", "dispatch_failed"])
                    .inc();
                StatusCode::BAD_REQUEST
            }
        }
    }
}
