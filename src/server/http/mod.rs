use crate::config::environment::ENV;
use crate::server::http::handlers::create_application_routes;
use tracing_subscriber;

pub mod handlers;

pub async fn run(port: &str) {
    tracing_subscriber::fmt::init();

    tracing::info!("Preparing application router...");

    let application_router = create_application_routes();

    tracing::info!("Application routes have been created successfully!");
    tracing::info!("Starting listener on 127.0.0.1:{}...", port);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap_or_else(|err| {
            panic!("Failed to bind TCP listener on port {}: {}", port, err);
        });

    tracing::info!("Server started successfully on 127.0.0.1:{}", port);

    let secret = ENV.get_or("GITHUB_WEBHOOK_SECRET", "");

    if secret == "" {
        tracing::warn!(
            "GITHUB_WEBHOOK_SECRET is empty. If you use this in production more please, add this environment"
        );
    }

    axum::serve(listener, application_router)
        .await
        .unwrap_or_else(|err| {
            panic!("Server error: {}", err);
        });
}
