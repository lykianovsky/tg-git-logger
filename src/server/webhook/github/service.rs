use crate::server::notifier::NotifierService;
use crate::server::webhook::github::events::GithubEvent;
use crate::server::webhook::github::pull_request::payload::PullRequestEvent;
use crate::server::webhook::github::push::payload::PushEvent;
use crate::utils::notifier::message_builder::MessageBuilder;
use axum::http::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

pub struct GithubWebhookService {
    notifier: Arc<NotifierService>,
}

impl GithubWebhookService {
    pub fn new(notifier: Arc<NotifierService>) -> Self {
        Self { notifier }
    }

    pub fn handle<Event>(&self, payload: Value) -> StatusCode
    where
        Event: GithubEvent + DeserializeOwned + Debug,
    {
        match serde_json::from_value::<Event>(payload) {
            Ok(event) => {
                self.notifier.notify_async(&event.build());
                StatusCode::NO_CONTENT
            }
            Err(e) => {
                tracing::error!("Failed to parse event: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
