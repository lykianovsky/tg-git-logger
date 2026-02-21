use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OAuthClientExchangeCodeError {
    #[error("Unsupported social type: {0}")]
    UnsupportedSocialType(String),

    #[error("Network error: {0}")]
    Transport(String),
}

#[derive(Deserialize, Debug)]
pub struct OAuthClientExchangeCodeResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

#[async_trait]
pub trait OAuthClient: Send + Sync {
    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<OAuthClientExchangeCodeResponse, OAuthClientExchangeCodeError>;
}
