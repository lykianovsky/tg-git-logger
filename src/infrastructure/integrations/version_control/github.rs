use crate::config::environment::ENV;
use crate::domain::auth::ports::oauth_client::{
    OAuthClientExchangeCodeError, OAuthClientExchangeCodeResponse,
};
use crate::domain::user::ports::version_control_client::{
    VersionControlClient, VersionControlClientError, VersionControlClientUser,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
}

pub struct GithubVersionControlClient {
    base: String,
    client: Client,
}

impl GithubVersionControlClient {
    pub fn new() -> Self {
        Self {
            // TODO
            base: ENV.get("GITHUB_API_BASE"),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl VersionControlClient for GithubVersionControlClient {
    async fn get_user(
        &self,
        access_token: &str,
    ) -> Result<VersionControlClientUser, VersionControlClientError> {
        let resp = self
            .client
            .get(format!("{}/user", self.base))
            .bearer_auth(access_token)
            .header("User-Agent", "Telegram-Git-App")
            .send()
            .await
            .map_err(|e| VersionControlClientError::Transport(e.to_string()))?;

        let body_text = resp
            .text()
            .await
            .map_err(|e| VersionControlClientError::Transport(e.to_string()))?;

        tracing::debug!("GitHub get_user response body: {}", body_text);

        let user = serde_json::from_str::<GithubUser>(&body_text).map_err(|e| {
            tracing::error!("Failed to parse GitHub OAuth response: {}", e);
            VersionControlClientError::Transport(e.to_string())
        })?;

        Ok(VersionControlClientUser {
            id: user.id,
            email: user.email,
            login: user.login,
        })
    }
}
