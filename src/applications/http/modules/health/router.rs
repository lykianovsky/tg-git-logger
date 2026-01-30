use crate::applications::http::modules::health::handler::HealthHandler;
use crate::applications::state::ApplicationState;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

pub struct HealthRouter {}

impl HealthRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating health router");

        Router::new()
            .route("/", get(HealthHandler::handle))
    }
}