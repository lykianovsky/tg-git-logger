mod consumers;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::rabbitmq::consumers::webhook_events::pull_request::RabbitMQWebhookPullRequestConsumer;
use crate::delivery::events::rabbitmq::consumers::webhook_events::push::RabbitMQWebhookPushConsumer;
use crate::delivery::events::rabbitmq::consumers::webhook_events::release::RabbitMQWebhookReleaseConsumer;
use crate::delivery::events::rabbitmq::consumers::webhook_events::workflow::RabbitMQWebhookWorkflowConsumer;
use crate::domain::shared::events::consumer_runner::EventConsumerRunner;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::rabbitmq::connector::RabbitMQConnector;
use crate::infrastructure::drivers::message_broker::rabbitmq::consumer_runner::RabbitMQEventConsumerRunner;
use async_trait::async_trait;
use consumers::user_registered::send_notification::RabbitMQUserRegisteredSendNotificationConsumer;
use std::error::Error;
use std::sync::Arc;

pub struct DeliveryRabbitMqEvents {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    connection: Arc<RabbitMQConnector>,
}

impl DeliveryRabbitMqEvents {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
        connection: Arc<RabbitMQConnector>,
    ) -> Self {
        Self {
            executors,
            config,
            connection,
        }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryRabbitMqEvents {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let social_type = SocialType::Telegram;
        let chat_id = SocialChatId(self.config.telegram.chat_id);
        RabbitMQEventConsumerRunner::new(self.connection.clone())
            .register(RabbitMQUserRegisteredSendNotificationConsumer {})
            .register(RabbitMQWebhookPullRequestConsumer {
                social_type: social_type.clone(),
                chat_id: chat_id.clone(),
                notify_received_webhook_event: self
                    .executors
                    .commands
                    .notify_received_webhook_event
                    .clone(),
            })
            .register(RabbitMQWebhookPushConsumer {
                social_type: social_type.clone(),
                chat_id: chat_id.clone(),
                notify_received_webhook_event: self
                    .executors
                    .commands
                    .notify_received_webhook_event
                    .clone(),
            })
            .register(RabbitMQWebhookReleaseConsumer {
                social_type: social_type.clone(),
                chat_id: chat_id.clone(),
                notify_received_webhook_event: self
                    .executors
                    .commands
                    .notify_received_webhook_event
                    .clone(),
            })
            .register(RabbitMQWebhookWorkflowConsumer {
                social_type: social_type.clone(),
                chat_id: chat_id.clone(),
                notify_received_webhook_event: self
                    .executors
                    .commands
                    .notify_received_webhook_event
                    .clone(),
            })
            .run()
            .await
            .map_err(|e| Box::<dyn Error>::from(e))?;

        Ok(())
    }
}
