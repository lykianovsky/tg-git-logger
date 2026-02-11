use crate::app::github::GithubRouter;
use crate::app::health::router::HealthRouter;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use std::sync::Arc;

pub struct ApplicationHttpRoutes {}

impl ApplicationHttpRoutes {
    pub fn build(application_state: Arc<ApplicationState>) -> Router {
        tracing::debug!("Start generate routing for http application");

        let health_route = HealthRouter::create();
        let github_route = GithubRouter::create();

        return Router::new()
            .merge(health_route)
            .nest("/github", github_route)
            .with_state(application_state);
    }
}
