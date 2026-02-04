mod routes;
mod middlewares;
mod modules;

use crate::applications::http::routes::ApplicationHttpRoutes;
use crate::applications::state::ApplicationState;
use crate::config::environment::ENV;
use axum::Router;
use std::sync::Arc;

pub async fn run(application_state: Arc<ApplicationState>) {
    let port = ENV.get("APPLICATION_PORT");

    tracing::info!("Preparing application router...");

    let application_router: Router = ApplicationHttpRoutes::build(application_state);

    tracing::info!("Application routes have been created successfully!");
    tracing::info!("Starting listener on {} port", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap_or_else(|err| {
            panic!("Failed to bind TCP listener on port {}: {}", port, err);
        });

    tracing::info!("Server started successfully on {} port", port);

    axum::serve(listener, application_router)
        .await
        .unwrap_or_else(|err| {
            panic!("Server error: {}", err);
        });
}
