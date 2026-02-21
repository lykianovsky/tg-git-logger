use crate::domain::shared::events::publisher::EventPublisherError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchWebhookEventExecutorError {
    #[error("{0}")]
    PublisherError(#[from] EventPublisherError),
}
