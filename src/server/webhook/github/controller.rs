use crate::server::webhook::github::events::GithubEventName;
use crate::server::webhook::github::service::GithubWebhookService;
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
        let raw_event_type = match headers
            .get(GithubHeaders::EVENT)
            .and_then(|header_value| header_value.to_str().ok())
        {
            Some(v) => v,
            None => {
                tracing::error!("Failed to read {} from GitHub header", GithubHeaders::EVENT);
                return StatusCode::FORBIDDEN;
            }
        };

        tracing::info!("Got event from webhook: {}", raw_event_type);

        let event_name = GithubEventName::from_str(raw_event_type)
            .unwrap_or(GithubEventName::Unknown(raw_event_type.to_string()));

        let payload: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => return StatusCode::BAD_REQUEST,
        };

        match event_name {
            GithubEventName::Push => self.service.handle_push(payload),
            GithubEventName::PullRequest => StatusCode::NO_CONTENT,
            GithubEventName::Issues => StatusCode::NO_CONTENT,
            GithubEventName::Release => StatusCode::NO_CONTENT,
            GithubEventName::Ping => StatusCode::NO_CONTENT,
            GithubEventName::Unknown(_) => StatusCode::FORBIDDEN,
        }
    }
}

pub async fn handle(
    State(controller): State<Arc<GithubWebhookController>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    controller.handle(headers, body)
}
