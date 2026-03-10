use crate::infrastructure::drivers::message_broker::contracts::acknowledger::BrokerMessageAcknowledger;
use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueueRetryPolicy;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::{
    MessageBrokerRabbitMQ, RoutingKeys, EXCHANGE_NAME,
};
use async_trait::async_trait;
use lapin::options::{BasicAckOptions, BasicNackOptions, BasicPublishOptions};
use lapin::types::AMQPValue;
use lapin::Channel;
use std::sync::Arc;

pub struct RabbitMQAcknowledger {
    pub delivery: Arc<lapin::message::Delivery>,
    pub channel: Arc<Channel>,
    pub routing_keys: RoutingKeys,
    pub retry_policy: Option<MessageBrokerQueueRetryPolicy>,
}

#[async_trait]
impl BrokerMessageAcknowledger for RabbitMQAcknowledger {
    async fn ack(&self) {
        self.delivery.ack(BasicAckOptions::default()).await.ok();
    }

    async fn requeue(&self) {
        self.delivery
            .nack(BasicNackOptions {
                requeue: true,
                ..Default::default()
            })
            .await
            .ok();
    }

    async fn retry(&self, reason: &str) {
        let mut headers = self
            .delivery
            .properties
            .headers()
            .clone()
            .unwrap_or_default();

        let mut history: Vec<serde_json::Value> = headers
            .inner()
            .get("x-error-history")
            .and_then(|v| match v {
                AMQPValue::LongString(s) => serde_json::from_str(s.to_string().as_str()).ok(),
                _ => None,
            })
            .unwrap_or_default();

        // TODO: Сделать структуру
        history.push(serde_json::json!({
            "reason": reason,
            "at": chrono::Utc::now()
        }));

        // TODO: Сделать константу
        headers.insert(
            "x-error-history".into(),
            AMQPValue::LongString(serde_json::to_string(&history).unwrap().into()),
        );

        let attempts = MessageBrokerRabbitMQ::get_retry_attempts(&self.delivery);

        let mut properties = self.delivery.properties.clone().with_headers(headers);

        if let Some(retry_policy) = self.retry_policy.clone() {
            let new_expiration = retry_policy.delay_ms * 2i64.pow(attempts as u32);
            properties = properties.with_expiration(new_expiration.to_string().into())
        }

        self.channel
            .basic_publish(
                EXCHANGE_NAME,
                &self.routing_keys.retry,
                Default::default(),
                &self.delivery.data,
                properties,
            )
            .await
            .unwrap();

        self.delivery.ack(Default::default()).await.ok();
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
                &self.routing_keys.dead,
                BasicPublishOptions::default(),
                &self.delivery.data,
                properties,
            )
            .await
            .ok();

        self.delivery.ack(BasicAckOptions::default()).await.ok();
    }
}
