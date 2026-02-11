use crate::app::github::auth::callback::handler::GithubAuthorizationCallbackHandler;
use crate::infrastructure::delivery::state::ApplicationState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub struct GithubAuthorizationCallbackRouter {}

impl GithubAuthorizationCallbackRouter {
    pub fn create() -> Router<Arc<ApplicationState>> {
        tracing::debug!("Creating github authorization callback router");

        Router::new().route("/callback", get(GithubAuthorizationCallbackHandler::handle))
    }
}
