mod webhook;

use crate::applications::http::routes;
use crate::applications::state::ApplicationState;
use axum::Router;
use std::sync::Arc;

pub fn build(application_state: Arc<ApplicationState>) -> Router {
    let github_webhook_route = routes::webhook::github::create();

    return Router::new()
        .merge(github_webhook_route)
        .with_state(application_state);
}