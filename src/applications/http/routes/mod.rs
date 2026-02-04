use crate::applications::http::modules::health::router::HealthRouter;
use crate::applications::http::modules::webhook::github::router::GithubWebhookRouter;
use crate::applications::state::ApplicationState;
use axum::Router;
use std::sync::Arc;

pub struct ApplicationHttpRoutes {}

impl ApplicationHttpRoutes {
    pub fn build(application_state: Arc<ApplicationState>) -> Router {
        tracing::debug!("Start generate routing for http application");

        let health_route = HealthRouter::create();
        let github_webhook_route = GithubWebhookRouter::create();

        return Router::new()
            .merge(health_route)
            .merge(github_webhook_route)
            .with_state(application_state);
    }
}

