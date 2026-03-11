pub mod github;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::listeners::github::webhook::pull_request::WebhookPullRequestEventListener;
use crate::delivery::events::listeners::github::webhook::push::WebhookPushEventListener;
use crate::delivery::events::listeners::github::webhook::release::WebhookReleaseEventListener;
use crate::delivery::events::listeners::github::webhook::workflow::WebhookWorkflowEventListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

pub struct DeliveryEventListeners {
    shared_dependency: Arc<ApplicationSharedDependency>,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryEventListeners {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
        shared_dependency: Arc<ApplicationSharedDependency>,
    ) -> Self {
        Self {
            executors,
            config,
            shared_dependency,
        }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryEventListeners {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        self.shared_dependency
            .event_bus
            .on(WebhookPullRequestEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                chat_id: SocialChatId(self.config.telegram.chat_id),
                task_tracker_service: self.shared_dependency.task_tracker_service.clone(),
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookPushEventListener {
                publisher: self.shared_dependency.publisher.clone(),
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookReleaseEventListener {
                publisher: self.shared_dependency.publisher.clone(),
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookWorkflowEventListener {
                publisher: self.shared_dependency.publisher.clone(),
            })
            .await;

        Ok(())
    }
}
