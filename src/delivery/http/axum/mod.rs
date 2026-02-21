mod controllers;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::http::axum::controllers::oauth::github::AxumOAuthGithubController;
use crate::delivery::http::axum::controllers::webhook::github::AxumWebhookGithubController;
use axum::routing::post;
use axum::{routing::get, Extension, Router};
use std::sync::Arc;

pub struct DeliveryHttpServerAxum {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    router: Router,
}

impl DeliveryHttpServerAxum {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        let router = Router::new()
            .route("/ping", get(|| async { "PONG" }))
            .nest(
                "/oauth",
                Router::new().route(
                    "/github",
                    get(AxumOAuthGithubController::handle_post).layer(Extension(
                        executors.commands.register_user_via_oauth.clone(),
                    )),
                ),
            )
            .nest(
                "/webhook",
                Router::new().route(
                    "/github",
                    post(AxumWebhookGithubController::handle_post)
                        .layer(Extension(executors.commands.dispatch_webhook_event.clone())),
                ),
            );

        Self {
            executors,
            config,
            router,
        }
    }
}

#[async_trait::async_trait]
impl ApplicationDelivery for DeliveryHttpServerAxum {
    async fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let port = self.config.port;

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|err| {
                eprintln!("Failed to bind TCP listener on port {}: {}", port, err);
                err
            })?;

        axum::serve(listener, self.router.clone())
            .await
            .map_err(|err| {
                eprintln!("Server error: {}", err);
                err
            })?;

        Ok(())
    }
}
