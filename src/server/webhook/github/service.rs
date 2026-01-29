use crate::client::task_tracker::TaskTrackerClient;
use crate::config::environment::ENV;
use crate::server::notifier::NotifierService;
use crate::server::task_tracker::TaskTrackerService;
use crate::server::webhook::github::events::GithubEvent;
use crate::server::webhook::github::pull_request::handler::PullRequestHandler;
use crate::server::webhook::github::pull_request::payload::PullRequestEvent;
use crate::server::webhook::github::push::payload::PushEvent;
use crate::utils::notifier::message_builder::MessageBuilder;
use axum::http::StatusCode;
use reqwest::Error;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::any::{type_name, Any, TypeId};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::event;

pub struct GithubWebhookService {
    notifier: Arc<NotifierService>,
    task_tracker: Arc<TaskTrackerService>,
}

impl GithubWebhookService {
    pub fn new(notifier: Arc<NotifierService>, task_tracker: Arc<TaskTrackerService>) -> Self {
        Self {
            notifier,
            task_tracker,
        }
    }

    pub fn handle<Event>(&self, payload: Value) -> StatusCode
    where
        Event: GithubEvent + DeserializeOwned + Debug + 'static,
    {
        let event_name = type_name::<Event>();

        match serde_json::from_value::<Event>(payload) {
            Ok(event) => {
                let event_any = &event as &dyn Any;
                if let Some(pr) = event_any.downcast_ref::<PullRequestEvent>() {
                    let pr_handler = PullRequestHandler::new(
                        Arc::clone(&self.task_tracker),
                        Arc::clone(&self.notifier),
                    );
                    pr_handler.handle(pr);
                }

                tracing::debug!(event = event_name, "Received GitHub event");

                let message = Arc::new(event.build());
                self.notifier.notify_async(message);
                StatusCode::NO_CONTENT
            }
            Err(e) => {
                tracing::error!(
                    event = event_name,
                    error = %e,
                    "Failed to parse event"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
