use crate::config::environment::ENV;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GithubOAuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse GitHub token: {0}")]
    FailedParse(#[from] serde_json::Error),
}

#[derive(Deserialize, Debug)]
pub struct GithubTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

pub struct GithubOAuthClient {
    base: String,
    client_id: String,
    client_secret: String,
    client: Client,
}

impl GithubOAuthClient {
    pub fn new() -> Self {
        Self {
            base: ENV.get("GITHUB_BASE"),
            client_id: ENV.get("GITHUB_OAUTH_CLIENT_ID"),
            client_secret: ENV.get("GITHUB_OAUTH_CLIENT_SECRET"),
            client: Client::new(),
        }
    }

    pub async fn get_access_token(
        &self,
        code: &str,
    ) -> Result<GithubTokenResponse, GithubOAuthError> {
        tracing::info!("Starting verification github user with code: {code}");

        let resp = self
            .client
            .post(format!("{}/login/oauth/access_token", self.base))
            .header("Accept", "application/json")
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
            ])
            .send()
            .await?
            .error_for_status()?;

        // Сначала читаем как текст
        let body_text = resp.text().await?;
        tracing::debug!("GitHub OAuth response body: {}", body_text);

        match serde_json::from_str::<GithubTokenResponse>(&body_text) {
            Ok(token) => Ok(token),
            Err(e) => {
                tracing::error!("Failed to decode GitHub token: {} | body: {}", e, body_text);
                Err(GithubOAuthError::FailedParse(e))
            }
        }
    }
}
