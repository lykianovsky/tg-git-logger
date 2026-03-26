use crate::application::user::commands::register_via_oauth::command::RegisterUserViaOAuthExecutorCommand;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::events::registration_failed::UserRegistrationFailedEvent;
use crate::domain::user::events::registration_success::UserRegistrationSuccessEvent;
use crate::infrastructure::drivers::cache::contract::CacheService;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::Extension;
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
enum RetrieveOAuthStateError {
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Invalid state error")]
    InvalidState,
    #[error("{0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
pub struct AxumOAuthGithubControllerPostQuery {
    code: String,
    state: String,
}

pub struct AxumOAuthGithubController {}

impl AxumOAuthGithubController {
    async fn retrieve_oauth_state(
        key: &str,
        cache: Arc<dyn CacheService>,
    ) -> Result<OpenAuthorizationState, RetrieveOAuthStateError> {
        tracing::debug!(state_id = %key, "Retrieving OAuth state from cache");

        let state_json = cache
            .take(key)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    state_id = %key,
                    "Failed to retrieve state from cache"
                );
                RetrieveOAuthStateError::Cache(e.to_string())
            })?
            .ok_or_else(|| {
                tracing::warn!(
                    state_id = %key,
                    "OAuth state not found in cache (expired or invalid)"
                );
                RetrieveOAuthStateError::InvalidState
            })?;

        tracing::trace!(
            state_id = %key,
            state_json_length = state_json.len(),
            "OAuth state JSON retrieved from cache"
        );

        Ok(serde_json::from_str::<OpenAuthorizationState>(&state_json)?)
    }

    pub async fn handle_post(
        Extension(executor): Extension<Arc<RegisterUserViaOAuthExecutor>>,
        Extension(shared): Extension<Arc<ApplicationSharedDependency>>,
        Extension(config): Extension<Arc<ApplicationConfig>>,
        Query(query): Query<AxumOAuthGithubControllerPostQuery>,
    ) -> impl IntoResponse {
        let bot_url = config.telegram.bot_url.as_str();
        let key = query.state.clone();

        let state = match Self::retrieve_oauth_state(&key, shared.cache.clone()).await {
            Ok(s) => s,
            Err(..) => return Redirect::to(bot_url),
        };

        let cmd = RegisterUserViaOAuthExecutorCommand {
            code: query.code.clone(),
            state,
        };

        match executor.execute(&cmd).await {
            Ok(result) => {
                tracing::debug!("{:?} {:?}", result, query);
                shared
                    .publisher
                    .publish(&UserRegistrationSuccessEvent {
                        chat_id: cmd.state.social_chat_id,
                        social_type: cmd.state.social_type,
                        user: result.user,
                        user_social_account: result.user_social_account,
                        user_version_control_account: result.user_version_control_account,
                    })
                    .await
                    .ok();
            }
            Err(error) => {
                tracing::error!("{:?} {:?}", error, query);
                shared
                    .publisher
                    .publish(&UserRegistrationFailedEvent {
                        chat_id: cmd.state.social_chat_id,
                        social_type: cmd.state.social_type,
                    })
                    .await
                    .ok();
            }
        }

        Redirect::to(bot_url)
    }
}
