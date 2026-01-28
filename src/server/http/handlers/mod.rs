mod health_check;

use crate::server::webhook;
use axum::{routing::get, Router};

pub fn create_application_routes() -> Router<()> {
    let webhook_router = webhook::github::create_router();

    Router::new()
        .route("/", get(health_check::check))
        .merge(webhook_router)
}
