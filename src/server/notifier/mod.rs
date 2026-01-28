use crate::utils::notifier::message_builder::MessageBuilder;
use crate::utils::notifier::Notifier;
use std::sync::Arc;

#[derive(Clone)]
pub struct NotifierService {
    client: Arc<dyn Notifier>,
}

impl NotifierService {
    pub fn new(client: Arc<dyn Notifier>) -> Self {
        Self { client }
    }

    pub fn notify_async(&self, message: Arc<MessageBuilder>) {
        let client = Arc::clone(&self.client);
        let message = Arc::clone(&message);

        tokio::spawn(async move {
            if let Err(e) = client.notify(&message).await {
                tracing::error!("Failed to send notification: {}", e);
            }
        });
    }
}
