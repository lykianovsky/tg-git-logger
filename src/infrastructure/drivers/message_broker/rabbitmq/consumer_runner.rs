use async_trait::async_trait;
use futures::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions,
        ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    }, types::{AMQPValue, FieldTable},
    BasicProperties,
    Channel,
};
use std::sync::Arc;

use crate::domain::shared::events::consumer::{EventConsumer, EventConsumerError};
use crate::domain::shared::events::consumer_runner::EventConsumerRunner;
use crate::domain::shared::events::retry_policy::RetryPolicy;
use crate::infrastructure::drivers::message_broker::rabbitmq::connector::RabbitMQConnector;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::{
    EXCHANGE_KIND, EXCHANGE_NAME,
};

// ── Erased consumer ────────────────────────────────────────────────────────────

#[async_trait]
pub trait ErasedEventConsumer: Send + Sync {
    fn routing_key(&self) -> &'static str;
    fn queue_name(&self) -> &'static str;
    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::default()
    }
    async fn handle_raw(&self, payload: &[u8]) -> Result<(), EventConsumerError>;
}

pub struct EventConsumerWrapper<C: EventConsumer> {
    consumer: C,
}

#[async_trait]
impl<C: EventConsumer + Send + Sync> ErasedEventConsumer for EventConsumerWrapper<C> {
    fn routing_key(&self) -> &'static str {
        self.consumer.routing_key()
    }

    fn queue_name(&self) -> &'static str {
        self.consumer.queue_name()
    }

    fn retry_policy(&self) -> RetryPolicy {
        self.consumer.retry_policy()
    }

    async fn handle_raw(&self, payload: &[u8]) -> Result<(), EventConsumerError> {
        let event = serde_json::from_slice(payload)
            .map_err(|e| EventConsumerError::DeserializationFailed(e.to_string()))?;
        self.consumer.handle(event).await
    }
}

// ── Queue names ────────────────────────────────────────────────────────────────

struct QueueNames {
    main: String,
    retry: String,
    dead: String,
}

struct RoutingKeys {
    main: String,
    retry: String,
    dead: String,
}

impl QueueNames {
    fn from(queue_name: &str) -> Self {
        Self {
            main: queue_name.to_string(),
            retry: format!("{}.retry", queue_name),
            dead: format!("{}.dead", queue_name),
        }
    }
}

impl RoutingKeys {
    fn from(routing_key: &str) -> Self {
        Self {
            main: routing_key.to_string(),
            retry: format!("{}.retry", routing_key),
            dead: format!("{}.dead", routing_key),
        }
    }
}

// ── Runner ─────────────────────────────────────────────────────────────────────

pub struct RabbitMQEventConsumerRunner {
    connection: Arc<RabbitMQConnector>,
    consumers: Vec<Box<dyn ErasedEventConsumer>>,
}

impl RabbitMQEventConsumerRunner {
    pub fn new(connection: Arc<RabbitMQConnector>) -> Self {
        Self {
            connection,
            consumers: Vec::new(),
        }
    }

    // ── Queue setup ────────────────────────────────────────────────────────────

    async fn setup_queues(
        channel: &Channel,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
        retry_policy: &RetryPolicy,
    ) -> Result<(), lapin::Error> {
        Self::declare_exchange(channel).await?;
        Self::declare_main_queue(channel, queue_names, routing_keys).await?;
        Self::declare_retry_queue(channel, queue_names, routing_keys, retry_policy).await?;
        Self::declare_dead_queue(channel, queue_names, routing_keys).await?;
        Ok(())
    }

    async fn declare_exchange(channel: &Channel) -> Result<(), lapin::Error> {
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
            .await
    }

    async fn declare_main_queue(
        channel: &Channel,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
    ) -> Result<(), lapin::Error> {
        let mut args = FieldTable::default();
        args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(EXCHANGE_NAME.into()),
        );
        args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(routing_keys.retry.clone().into()),
        );

        channel
            .queue_declare(
                &queue_names.main,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                args,
            )
            .await?;

        channel
            .queue_bind(
                &queue_names.main,
                EXCHANGE_NAME,
                &routing_keys.main,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    async fn declare_retry_queue(
        channel: &Channel,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
        retry_policy: &RetryPolicy,
    ) -> Result<(), lapin::Error> {
        let mut args = FieldTable::default();
        args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(EXCHANGE_NAME.into()),
        );
        args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(routing_keys.main.clone().into()),
        );
        args.insert(
            "x-message-ttl".into(),
            AMQPValue::LongUInt(retry_policy.delay_ms),
        );

        channel
            .queue_declare(
                &queue_names.retry,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                args,
            )
            .await?;

        channel
            .queue_bind(
                &queue_names.retry,
                EXCHANGE_NAME,
                &routing_keys.retry,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    async fn declare_dead_queue(
        channel: &Channel,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
    ) -> Result<(), lapin::Error> {
        channel
            .queue_declare(
                &queue_names.dead,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        channel
            .queue_bind(
                &queue_names.dead,
                EXCHANGE_NAME,
                &routing_keys.dead,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    // ── Retry helpers ──────────────────────────────────────────────────────────

    fn get_retry_attempts(delivery: &lapin::message::Delivery) -> i64 {
        delivery
            .properties
            .headers()
            .as_ref()
            .and_then(|h| h.inner().get("x-death"))
            .and_then(|v| match v {
                AMQPValue::FieldArray(arr) => arr.as_slice().first().cloned(),
                _ => None,
            })
            .and_then(|v| match v {
                AMQPValue::FieldTable(t) => t.inner().get("count").cloned(),
                _ => None,
            })
            .and_then(|v| match v {
                AMQPValue::LongLongInt(n) => Some(n),
                _ => None,
            })
            .unwrap_or(0)
    }

    fn build_dead_letter_properties(error: &str) -> BasicProperties {
        let mut headers = FieldTable::default();
        headers.insert("x-error-reason".into(), AMQPValue::LongString(error.into()));
        headers.insert(
            "x-failed-at".into(),
            AMQPValue::LongString(chrono::Utc::now().to_rfc3339().into()),
        );
        BasicProperties::default().with_headers(headers)
    }

    async fn publish_to_dead(
        channel: &Channel,
        dead_routing_key: &str,
        payload: &[u8],
        error: &str,
    ) {
        let _ = channel
            .basic_publish(
                EXCHANGE_NAME,
                dead_routing_key,
                BasicPublishOptions::default(),
                payload,
                Self::build_dead_letter_properties(error),
            )
            .await;
    }

    // ── Consume loop ───────────────────────────────────────────────────────────

    async fn consume_loop(
        channel: Channel,
        queue_name: &'static str,
        routing_key: &'static str,
        consumer: Box<dyn ErasedEventConsumer>,
        retry_policy: RetryPolicy,
    ) {
        let mut stream = match channel
            .basic_consume(
                queue_name,
                &format!("{queue_name}.consumer"),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
        {
            Ok(stream) => stream,
            Err(e) => {
                tracing::error!(queue_name, routing_key, error = %e, "Failed to start consumer");
                return;
            }
        };

        tracing::info!(queue_name, routing_key, "Consumer listening");

        let dead_routing_key = format!("{}.dead", routing_key);

        while let Some(delivery_result) = stream.next().await {
            let delivery = match delivery_result {
                Ok(delivery) => delivery,
                Err(e) => {
                    tracing::warn!(queue_name, error = %e, "Delivery error — exiting loop");
                    break;
                }
            };

            match consumer.handle_raw(&delivery.data).await {
                Ok(()) => {
                    if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                        tracing::error!(queue_name, error = %e, "Failed to ack message");
                    }
                }

                Err(EventConsumerError::DeserializationFailed(e)) => {
                    tracing::error!(
                        queue_name,
                        error = %e,
                        "Deserialization failed — message is unrecoverable, moving to dead queue"
                    );
                    Self::publish_to_dead(&channel, &dead_routing_key, &delivery.data, &e).await;
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }

                Err(e) => {
                    let attempts = Self::get_retry_attempts(&delivery);

                    if attempts >= retry_policy.max_attempts {
                        tracing::error!(
                            queue_name,
                            attempts,
                            max = retry_policy.max_attempts,
                            error = %e,
                            "Max retries exceeded — moving to dead queue"
                        );
                        Self::publish_to_dead(
                            &channel,
                            &dead_routing_key,
                            &delivery.data,
                            &e.to_string(),
                        )
                        .await;
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                    } else {
                        tracing::warn!(
                            queue_name,
                            attempt = attempts + 1,
                            max = retry_policy.max_attempts,
                            delay_ms = retry_policy.delay_ms,
                            error = %e,
                            "Handler failed — retrying after delay"
                        );
                        let _ = delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await;
                    }
                }
            }
        }

        tracing::warn!(queue_name, "Consumer stream ended");
    }
}

// ── EventConsumerRunner impl ───────────────────────────────────────────────────

#[async_trait]
impl EventConsumerRunner for RabbitMQEventConsumerRunner {
    fn register<C: EventConsumer + Send + Sync + 'static>(mut self, consumer: C) -> Self {
        self.consumers
            .push(Box::new(EventConsumerWrapper { consumer }));
        self
    }

    async fn run(self) -> Result<(), String> {
        for consumer in self.consumers {
            let routing_key = consumer.routing_key();
            let queue_name = consumer.queue_name();
            let retry_policy = consumer.retry_policy();

            let queue_names = QueueNames::from(queue_name);
            let routing_keys = RoutingKeys::from(routing_key);

            let channel = self
                .connection
                .create_channel()
                .await
                .map_err(|e| e.to_string())?;

            Self::setup_queues(&channel, &queue_names, &routing_keys, &retry_policy)
                .await
                .map_err(|e| e.to_string())?;

            tokio::spawn(Self::consume_loop(
                channel,
                queue_name,
                routing_key,
                consumer,
                retry_policy,
            ));
        }

        tracing::info!("All consumers started");
        Ok(())
    }
}
