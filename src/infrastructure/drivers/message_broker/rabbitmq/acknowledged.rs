use crate::infrastructure::drivers::message_broker::contracts::acknowledger::BrokerMessageAcknowledger;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::EXCHANGE_NAME;
use async_trait::async_trait;
use lapin::options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions};
use lapin::types::AMQPValue;
use lapin::Channel;
use std::sync::Arc;

pub struct RabbitMQAcknowledger {
    pub delivery: Arc<lapin::message::Delivery>,
    pub channel: Arc<Channel>,
    pub dead_routing_key: String,
}

#[async_trait]
impl BrokerMessageAcknowledger for RabbitMQAcknowledger {
    async fn ack(&self) {
        self.delivery.ack(BasicAckOptions::default()).await.ok();
    }

    async fn nack(&self, requeue: bool) {
        self.delivery
            .nack(BasicNackOptions {
                requeue,
                ..Default::default()
            })
            .await
            .ok();
    }

    async fn reject(&self, reason: &str) {
        let mut headers = self
            .delivery
            .properties
            .headers()
            .clone()
            .unwrap_or_default();

        headers.insert("x-dead-reason".into(), AMQPValue::LongString(reason.into()));
        headers.insert(
            "x-dead-at".into(),
            AMQPValue::LongString(chrono::Utc::now().to_rfc3339().into()),
        );

        let properties = self.delivery.properties.clone().with_headers(headers);

        self.channel
            .basic_publish(
                EXCHANGE_NAME,
                &self.dead_routing_key,
                BasicPublishOptions::default(),
                &self.delivery.data,
                properties,
            )
            .await
            .ok();

        self.delivery.ack(BasicAckOptions::default()).await.ok();
    }
}
