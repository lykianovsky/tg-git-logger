use crate::application::user::commands::register_via_oauth::command::RegisterUserViaOAuthExecutorCommand;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::domain::shared::command::CommandExecutor;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum::Extension;
use reqwest::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct AxumOAuthGithubControllerPostQuery {
    code: String,
    state: String,
}

pub struct AxumOAuthGithubController {}

impl AxumOAuthGithubController {
    pub async fn handle_post(
        Extension(executor): Extension<Arc<RegisterUserViaOAuthExecutor>>,
        Query(query): Query<AxumOAuthGithubControllerPostQuery>,
    ) -> impl IntoResponse {
        let cmd = RegisterUserViaOAuthExecutorCommand {
            code: query.code.clone(),
            state: query.state.clone(),
        };

        match executor.execute(&cmd).await {
            Ok(result) => {
                tracing::debug!("{:?} {:?}", result, query);
            }
            Err(error) => {
                tracing::error!("{:?} {:?}", error, query);
            }
        }

        // TODO: Вынести к конфиг
        Redirect::to("https://t.me/zb_git_log_bot")
    }
}
