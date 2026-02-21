use crate::domain::shared::events::publisher::EventPublisherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateOAuthLinkExecutorError {
    #[error("Exist registered social account with this social_user_id: {0}")]
    ExistRegisteredSocialAccountError(String),

    #[error("{0}")]
    PublisherError(#[from] EventPublisherError),

    #[error("{0}")]
    UrlParse(#[from] url::ParseError),

    #[error("{0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Cache has exist by key: {0}")]
    CacheHasExist(String),
}
