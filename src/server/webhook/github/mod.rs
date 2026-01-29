use crate::client::task_tracker::TaskTrackerClient;
use crate::client::telegram::bot::TelegramBot;
use crate::client::telegram::client::TelegramHttpClient;
use crate::config::environment::ENV;
use crate::server::notifier::NotifierService;
use crate::server::task_tracker::TaskTrackerService;
use crate::server::webhook::github::controller::GithubWebhookController;
use crate::server::webhook::github::service::GithubWebhookService;
use crate::utils::notifier::telegram::TelegramNotifierAdapter;
use crate::utils::notifier::Notifier;
use axum::Router;
use std::sync::Arc;

pub mod controller;
pub mod events;
pub mod middleware;
pub mod pull_request;
pub mod push;
pub mod release;
pub mod service;
mod workflow;

pub fn create_router() -> Router {
    let chat_id: i64 = ENV.get("TELEGRAM_CHAT_ID").parse().unwrap();

    let telegram_bot: Arc<dyn TelegramBot> = Arc::new(TelegramHttpClient::new(
        ENV.get("TELEGRAM_URL_BASE"),
        ENV.get("TELEGRAM_BOT_TOKEN"),
    ));

    let telegram_adapter: Arc<dyn Notifier> =
        Arc::new(TelegramNotifierAdapter::new(telegram_bot, chat_id));

    let notifier = Arc::new(NotifierService::new(telegram_adapter));

    let task_tracker_client = Arc::new(TaskTrackerClient::new(
        ENV.get("TASK_TRACKER_BASE"),
        ENV.get("TASK_TRACKER_API_TOKEN"),
    ));

    let task_tracker = Arc::new(TaskTrackerService::new(task_tracker_client));

    let service = Arc::new(GithubWebhookService::new(notifier, task_tracker));
    let controller = Arc::new(GithubWebhookController::new(service));

    Router::new()
        .route("/webhook/github", axum::routing::post(controller::handle))
        .with_state(controller)
        .layer(axum::middleware::from_fn(middleware::handle))
}
