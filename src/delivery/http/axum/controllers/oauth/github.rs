use crate::application::user::commands::register_via_oauth::command::RegisterUserViaOAuthExecutorCommand;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use axum::extract::Query;
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
    ) -> StatusCode {
        let cmd = RegisterUserViaOAuthExecutorCommand {
            code: query.code.clone(),
            state: query.state.clone(),
        };

        match executor.execute(cmd).await {
            Ok(result) => {
                tracing::debug!("{:?} {:?}", result, query);
                StatusCode::OK
            }
            Err(error) => {
                tracing::error!("{:?} {:?}", error, query);
                StatusCode::BAD_REQUEST
            }
        }
    }
}
