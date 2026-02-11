use crate::app::github::auth::router::GithubAuthorizationRouter;
use crate::app::github::webhook::router::GithubWebhookRouter;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use std::sync::Arc;

mod auth;
mod webhook;

pub struct GithubRouter {}

impl GithubRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating github authorization callback router");

        let webhook = GithubWebhookRouter::create();
        let auth = GithubAuthorizationRouter::create();

        Router::new().merge(webhook).merge(auth)
    }
}
