use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyReceivedWebhookEventExecutorError {
    #[error("Unknown error occurred while processing the received webhook event")]
    UnknownError,
}
