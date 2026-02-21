use std::sync::Arc;

use crate::domain::shared::events::event::DomainEvent;
use crate::domain::shared::events::publisher::{EventPublisher, EventPublisherError};
use crate::infrastructure::drivers::message_broker::rabbitmq::connector::RabbitMQConnector;
use async_trait::async_trait;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties,
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
};
use tokio::sync::Mutex;
use tracing::error;

pub const EXCHANGE_NAME: &str = "domain_events";
pub const EXCHANGE_KIND: lapin::ExchangeKind = lapin::ExchangeKind::Topic;

pub struct RabbitMqPublisher {
    channel: Arc<Mutex<Channel>>,
}

impl RabbitMqPublisher {
    pub async fn new(connector: Arc<RabbitMQConnector>) -> Result<Self, lapin::Error> {
        let channel = connector.create_channel().await?;

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
impl EventPublisher for RabbitMqPublisher {
    async fn publish(&self, event: &dyn DomainEvent) -> Result<(), EventPublisherError> {
        let payload = serde_json::to_vec(event)
            .map_err(|e| EventPublisherError::SerializationFailed(e.to_string()))?;

        let routing_key = event.event_name();

        let channel = self.channel.lock().await;

        let confirm = channel
            .basic_publish(
                EXCHANGE_NAME,
                routing_key,
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2), // persistent
            )
            .await
            .map_err(|e| {
                error!(
                    routing_key,
                    error = %e,
                    "Failed to publish domain event"
                );
                EventPublisherError::PublishFailed(e.to_string())
            })?;

        confirm.await.map_err(|e| {
            error!(routing_key, error = %e, "Broker rejected event");
            EventPublisherError::PublishFailed(e.to_string())
        })?;

        tracing::debug!(routing_key, "Domain event published");

        Ok(())
    }
}
