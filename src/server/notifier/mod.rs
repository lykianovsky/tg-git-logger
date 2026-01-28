use crate::client::notifier::message_builder::MessageBuilder;
use crate::client::notifier::Notifier;
use std::sync::Arc;

#[derive(Clone)]
pub struct NotifierService {
    client: Arc<dyn Notifier>,
}

impl NotifierService {
    // Конструктор, как в TypeScript
    pub fn new(client: Arc<dyn Notifier>) -> Self {
        Self { client }
    }

    // Fire-and-forget версия (не ждем результата)
    pub fn notify_async(&self, message: &MessageBuilder) {
        let client = Arc::clone(&self.client);
        let message_string = message.clone();

        tokio::spawn(async move {
            if let Err(e) = client.notify(&message_string).await {
                eprintln!("Failed to send notification: {}", e);
            }
        });
    }
}
