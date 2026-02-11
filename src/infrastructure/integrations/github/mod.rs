use crate::config::environment::ENV;
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

pub mod oauth;

#[derive(Debug, Error)]
pub enum GithubClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
}

pub struct GithubClient {
    base: String,
    client: Client,
}

impl GithubClient {
    pub fn new() -> Self {
        Self {
            base: ENV.get("GITHUB_API_BASE"),
            client: Client::new(),
        }
    }

    pub async fn get_user(&self, access_token: &str) -> Result<GithubUser, GithubClientError> {
        let resp = self
            .client
            .get(format!("{}/user", self.base))
            .bearer_auth(access_token)
            .header("User-Agent", "Telegram-Git-App")
            .send()
            .await?
            .error_for_status()?;

        let user = resp.json::<GithubUser>().await?;
        Ok(user)
    }
}
