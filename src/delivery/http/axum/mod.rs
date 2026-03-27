mod controllers;
mod middlewares;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::http::axum::controllers::oauth::github::AxumOAuthGithubController;
use crate::delivery::http::axum::controllers::report::AxumReportController;
use crate::delivery::http::axum::controllers::webhook::github::AxumWebhookGithubController;
use crate::delivery::http::axum::middlewares::GithubWebhookAuthorizationMiddleware;
use axum::routing::post;
use axum::{routing::get, Extension, Router};
use std::sync::Arc;

pub struct DeliveryHttpServerAxum {
    executors: Arc<ApplicationBoostrapExecutors>,
    shared_dependency: Arc<ApplicationSharedDependency>,
    config: Arc<ApplicationConfig>,
    router: Router,
}

impl DeliveryHttpServerAxum {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        shared_dependency: Arc<ApplicationSharedDependency>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        let middleware_config = config.clone();
        let router = Self::build_router(
            &executors,
            &shared_dependency,
            &config,
            middleware_config,
        );

        Self {
            executors,
            shared_dependency,
            config,
            router,
        }
    }

    fn build_router(
        executors: &Arc<ApplicationBoostrapExecutors>,
        shared_dependency: &Arc<ApplicationSharedDependency>,
        config: &Arc<ApplicationConfig>,
        middleware_config: Arc<ApplicationConfig>,
    ) -> Router {
        let oauth_routes = Router::new().route(
            "/github",
            get(AxumOAuthGithubController::handle_post)
                .layer(Extension(executors.commands.register_user_via_oauth.clone())),
        );

        let webhook_routes = Router::new().route(
            "/github",
            post(AxumWebhookGithubController::handle_post)
                .layer(Extension(executors.commands.dispatch_webhook_event.clone()))
                .layer(axum::middleware::from_fn(move |req, next| {
                    let mw = GithubWebhookAuthorizationMiddleware::new(
                        middleware_config.github.webhook_secret.clone(),
                    );
                    async move { mw.handle(req, next).await }
                })),
        );

        Router::new()
            .route("/ping", get(|| async { "PONG" }))
            .route("/report/{token}", get(AxumReportController::handle_get))
            .nest("/oauth", oauth_routes)
            .nest("/webhook", webhook_routes)
            .layer(Extension(config.clone()))
            .layer(Extension(shared_dependency.clone()))
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
