use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug)]
pub enum OAuthClientExchangeCodeError {
    UnsupportedSocialType(String),
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
