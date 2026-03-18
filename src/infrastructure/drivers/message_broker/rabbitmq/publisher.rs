use crate::infrastructure::drivers::message_broker::contracts::envelope::MessageBrokerEnvelope;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerPublisher, MessageBrokerPublisherPublishError,
};
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::{
    EXCHANGE_KIND, EXCHANGE_NAME,
};
use async_trait::async_trait;
use lapin::options::{BasicPublishOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, Channel, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MessageBrokerRabbitMQPublisher {
    channel: Arc<Mutex<Channel>>,
}

impl MessageBrokerRabbitMQPublisher {
    pub async fn new(connection: Arc<Connection>) -> Result<Self, lapin::Error> {
        tracing::info!("Connection to rabbitmq and create publisher channel");

        let channel = connection.create_channel().await?;

        channel
            .exchange_declare(
                EXCHANGE_NAME,
                EXCHANGE_KIND,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        tracing::info!(exchange = EXCHANGE_NAME, "RabbitMQ exchange declared");

        Ok(Self {
            channel: Arc::new(Mutex::new(channel)),
        })
    }
}

#[async_trait]
impl MessageBrokerPublisher for MessageBrokerRabbitMQPublisher {
    async fn publish(
        &self,
        message: &dyn MessageBrokerMessage,
    ) -> Result<(), MessageBrokerPublisherPublishError> {
        tracing::debug!("Create publish message for: {}", message.name());

        let envelope = serde_json::to_vec(&MessageBrokerEnvelope {
            name: message.name().to_string(),
            payload: message,
        })
        .map_err(|e| MessageBrokerPublisherPublishError::Serialization(e.to_string()))?;

        tracing::debug!("Lock execution, and get publisher channel");

        let channel = self.channel.lock().await;

        channel
            .basic_publish(
                EXCHANGE_NAME,
                &message.kind().routing_key(),
                BasicPublishOptions::default(),
                &envelope,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| MessageBrokerPublisherPublishError::PublishCreation(e.to_string()))?
            .await
            .map_err(|e| MessageBrokerPublisherPublishError::PublishConfirmation(e.to_string()))?;

        tracing::debug!(event_name = %message.name(), "Publish message to RabbitMQ complete successfully");

        Ok(())
    }
}
