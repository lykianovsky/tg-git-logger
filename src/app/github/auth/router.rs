use crate::app::github::auth::callback::router::GithubAuthorizationCallbackRouter;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use std::sync::Arc;

pub struct GithubAuthorizationRouter {}

impl GithubAuthorizationRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating github authorization router");

        let callback = GithubAuthorizationCallbackRouter::create();

        Router::new().nest("/auth", callback)
    }
}
