use crate::app::health::handler::HealthHandler;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub struct HealthRouter {}

impl HealthRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating health router");

        Router::new().route("/", get(HealthHandler::handle))
    }
}
