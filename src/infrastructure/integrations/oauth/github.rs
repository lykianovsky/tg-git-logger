use crate::config::environment::ENV;
use crate::domain::auth::ports::oauth_client::{
    OAuthClient, OAuthClientExchangeCodeError, OAuthClientExchangeCodeResponse,
};
use async_trait::async_trait;
use reqwest::Client;

pub struct GithubOAuthClient {
    base: String,
    client_id: String,
    client_secret: String,
    client: Client,
}

impl GithubOAuthClient {
    pub fn new(base: String, client_id: String, client_secret: String) -> Self {
        Self {
            base,
            client_id,
            client_secret,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl OAuthClient for GithubOAuthClient {
    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<OAuthClientExchangeCodeResponse, OAuthClientExchangeCodeError> {
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
            .await
            .map_err(|e| OAuthClientExchangeCodeError::Transport(e.to_string()))?;

        // Сначала читаем как текст
        let body_text = resp
            .text()
            .await
            .map_err(|e| OAuthClientExchangeCodeError::Transport(e.to_string()))?;

        tracing::debug!("GitHub OAuth response body: {}", body_text);

        Ok(
            serde_json::from_str::<OAuthClientExchangeCodeResponse>(&body_text).map_err(|e| {
                tracing::error!("Failed to parse GitHub OAuth response: {}", e);
                OAuthClientExchangeCodeError::Transport(e.to_string())
            })?,
        )
    }
}
