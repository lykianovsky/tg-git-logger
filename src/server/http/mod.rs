use crate::config::environment::ENV;
use crate::server::http::handlers::create_application_routes;
use axum::Router;
use teloxide::prelude::Requester;
use teloxide::Bot;
use tracing_subscriber;

pub mod handlers;

pub async fn run(port: &str) {
    let is_debug_enabled: bool = ENV.get("DEBUG").parse().unwrap();

    let debug_filter: &str = if is_debug_enabled { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(debug_filter)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .with_target(true)
        .with_file(true)
        .with_line_number(is_debug_enabled)
        .init();

    tracing::info!("Preparing application router...");

    let application_router: Router = create_application_routes();

    tracing::info!("Application routes have been created successfully!");
    tracing::info!("Starting listener on {} port", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap_or_else(|err| {
            panic!("Failed to bind TCP listener on port {}: {}", port, err);
        });

    tracing::info!("Server started successfully on {} port", port);

    let secret = ENV.get_or("GITHUB_WEBHOOK_SECRET", "");

    if secret == "" {
        tracing::warn!(
            "!!! GITHUB_WEBHOOK_SECRET is empty. If you use this in production more please, add this environment !!!"
        );
    }

    axum::serve(listener, application_router)
        .await
        .unwrap_or_else(|err| {
            panic!("Server error: {}", err);
        });
}
