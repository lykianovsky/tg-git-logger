pub mod github;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::listeners::github::webhook::pull_request::WebhookPullRequestEventListener;
use crate::delivery::events::listeners::github::webhook::push::WebhookPushEventListener;
use crate::delivery::events::listeners::github::webhook::release::WebhookReleaseEventListener;
use crate::delivery::events::listeners::github::webhook::workflow::WebhookWorkflowEventListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::infrastructure::processing::event_bus::EventBus;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

pub struct DeliveryEventListeners {
    event_bus: Arc<EventBus>,
    publisher: Arc<dyn MessageBrokerPublisher>,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryEventListeners {
    pub fn new(
        event_bus: Arc<EventBus>,
        publisher: Arc<dyn MessageBrokerPublisher>,
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        Self {
            event_bus,
            publisher,
            executors,
            config,
        }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryEventListeners {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        self.event_bus
            .on(WebhookPullRequestEventListener {
                publisher: self.publisher.clone(),
                chat_id: SocialChatId(self.config.telegram.chat_id),
            })
            .await;
        self.event_bus
            .on(WebhookPushEventListener {
                publisher: self.publisher.clone(),
            })
            .await;
        self.event_bus
            .on(WebhookReleaseEventListener {
                publisher: self.publisher.clone(),
            })
            .await;
        self.event_bus
            .on(WebhookWorkflowEventListener {
                publisher: self.publisher.clone(),
            })
            .await;

        Ok(())
    }
}
