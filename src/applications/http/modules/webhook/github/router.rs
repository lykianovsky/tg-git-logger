use crate::applications::http::middlewares::github_webhook_authorization;
use crate::applications::http::middlewares::github_webhook_authorization::GithubWebhookAuthorizationMiddleware;
use crate::applications::http::modules::webhook::github::handler::GithubWebhookHandler;
use crate::applications::state::ApplicationState;
use crate::config::environment::ENV;
use axum::routing::post;
use axum::Router;
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
            .route("/webhook/github", post(GithubWebhookHandler::handle))
            .layer(axum::middleware::from_fn(GithubWebhookAuthorizationMiddleware::handle))
    }
}