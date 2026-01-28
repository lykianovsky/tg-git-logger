use crate::client::notifier::message_builder::MessageBuilder;
use crate::server::notifier::NotifierService;
use crate::server::webhook::github::push::payload::PushEvent;
use axum::http::StatusCode;
use serde_json::Value;
use std::sync::Arc;

pub struct GithubWebhookService {
    notifier: Arc<NotifierService>,
}

impl GithubWebhookService {
    pub fn new(notifier: Arc<NotifierService>) -> Self {
        Self { notifier }
    }

    pub fn handle_push(&self, payload: Value) -> StatusCode {
        match PushEvent::from_value(payload) {
            Ok(event) => {
                self.notifier.notify_async(&event.build());
                StatusCode::NO_CONTENT
            }
            Err(e) => {
                tracing::error!("Failed to parse push event: {}", e);

                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
