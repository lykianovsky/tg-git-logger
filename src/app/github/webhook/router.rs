use crate::app::github::webhook::handler::GithubWebhookHandler;
use crate::config::environment::ENV;
use crate::infrastructure::delivery::http::middlewares::github_webhook_authorization::GithubWebhookAuthorizationMiddleware;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;

pub struct GithubWebhookRouter {}

impl GithubWebhookRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating github webhook router");

        let secret = ENV.get_or("GITHUB_WEBHOOK_SECRET", "");

        if secret == "" {
            tracing::warn!(
                "!!! GITHUB_WEBHOOK_SECRET is empty. If you use this in production more please, add this environment !!!"
            );
        }

        Router::new()
            .route("/github/webhook", post(GithubWebhookHandler::handle))
            .layer(axum::middleware::from_fn(
                GithubWebhookAuthorizationMiddleware::handle,
            ))
    }
}
