use crate::infrastructure::drivers::message_broker::contracts::acknowledger::BrokerMessageAcknowledger;
use crate::infrastructure::drivers::message_broker::contracts::broker::{
    MessageBroker, MessageBrokerConsumeError, MessageBrokerSetupError,
};
use crate::infrastructure::drivers::message_broker::contracts::delivery::BrokerDelivery;
use crate::infrastructure::drivers::message_broker::contracts::envelope::MessageBrokerEnvelope;
use crate::infrastructure::drivers::message_broker::contracts::queue::{
    MessageBrokerQueue, MessageBrokerQueueRetryPolicy,
};
use crate::infrastructure::drivers::message_broker::contracts::queue_builder::MessageBrokerStream;
use crate::infrastructure::drivers::message_broker::rabbitmq::acknowledged::RabbitMQAcknowledger;
use async_trait::async_trait;
use futures::StreamExt;
use lapin::options::{
    BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::AMQPValue;
use lapin::{types::FieldTable, Channel, Connection, ConnectionProperties, ExchangeKind};
use std::sync::Arc;

pub const EXCHANGE_NAME: &str = "domain.exchange";
pub const EXCHANGE_KIND: lapin::ExchangeKind = lapin::ExchangeKind::Topic;

pub struct QueueNames {
    main: String,
    retry: String,
    dead: String,
}

pub struct RoutingKeys {
    pub main: String,
    pub retry: String,
    pub dead: String,
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

pub struct MessageBrokerRabbitMQ {
    pub connection: Arc<Connection>,
    channel: Channel,
}

impl MessageBrokerRabbitMQ {
    pub async fn new(url: &str) -> Result<Self, lapin::Error> {
        let connection =
            Arc::new(Connection::connect(&url, ConnectionProperties::default()).await?);

        let channel = connection.create_channel().await?;

        Ok(Self {
            connection,
            channel,
        })
    }

    async fn declare_exchange(&self, exchange_name: &str) -> Result<(), lapin::Error> {
        self.channel
            .exchange_declare(
                exchange_name,
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
        &self,
        exchange_name: &str,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
    ) -> Result<(), lapin::Error> {
        let mut main_args = FieldTable::default();

        main_args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(exchange_name.into()),
        );
        main_args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(routing_keys.retry.clone().into()),
        );

        self.channel
            .queue_declare(
                queue_names.main.as_str(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                main_args,
            )
            .await?;

        self.channel
            .queue_bind(
                queue_names.main.as_str(),
                exchange_name,
                routing_keys.main.as_str(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    async fn declare_retry_queue(
        &self,
        exchange_name: &str,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
    ) -> Result<(), lapin::Error> {
        let mut args = FieldTable::default();
        args.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(exchange_name.into()),
        );
        args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(routing_keys.main.clone().into()),
        );

        self.channel
            .queue_declare(
                &queue_names.retry,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                args,
            )
            .await?;

        self.channel
            .queue_bind(
                &queue_names.retry,
                exchange_name,
                &routing_keys.retry,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    async fn declare_dead_queue(
        &self,
        exchange_name: &str,
        queue_names: &QueueNames,
        routing_keys: &RoutingKeys,
    ) -> Result<(), lapin::Error> {
        self.channel
            .queue_declare(
                &queue_names.dead,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        self.channel
            .queue_bind(
                &queue_names.dead,
                exchange_name,
                &routing_keys.dead,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
    }

    pub fn get_retry_attempts(delivery: &lapin::message::Delivery) -> i64 {
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

    fn get_error_history(delivery: &lapin::message::Delivery) -> Vec<String> {
        delivery
            .properties
            .headers()
            .as_ref()
            // TODO: from const
            .and_then(|h| h.inner().get("x-error-history"))
            .and_then(|v| match v {
                AMQPValue::LongString(s) => {
                    serde_json::from_str::<Vec<serde_json::Value>>(s.to_string().as_str()).ok()
                }
                _ => None,
            })
            // TODO: from struct
            .map(|vec| {
                vec.into_iter()
                    .map(|v| {
                        v.get("reason")
                            .and_then(|r| r.as_str())
                            .unwrap_or("")
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    async fn setup_queue(
        &self,
        exchange_name: &str,
        queue: &MessageBrokerQueue,
    ) -> Result<(), lapin::Error> {
        let queue_names = QueueNames::from(&queue.name);
        let routing_keys = RoutingKeys::from(&queue.routing_key);

        self.declare_exchange(exchange_name).await?;

        self.declare_main_queue(exchange_name, &queue_names, &routing_keys)
            .await?;

        self.declare_retry_queue(exchange_name, &queue_names, &routing_keys)
            .await?;

        self.declare_dead_queue(exchange_name, &queue_names, &routing_keys)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl MessageBroker for MessageBrokerRabbitMQ {
    async fn consume<'a>(
        &'a self,
        worker_name: &str,
        queue: &'a MessageBrokerQueue,
    ) -> Result<MessageBrokerStream<'a>, MessageBrokerConsumeError> {
        let channel = self
            .connection
            .create_channel()
            .await
            .map_err(|e| MessageBrokerConsumeError::ChannelCreate(e.to_string()))?;

        let stream = channel
            .basic_consume(
                &queue.name,
                &format!("{worker_name}.{}.consumer", queue.name),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| MessageBrokerConsumeError::ChannelStreamCreate(e.to_string()))?;

        let retry_policy = queue.retry_policy.clone();
        let channel = Arc::new(channel);

        let mapped = stream.filter_map({
            move |delivery| {
                let retry_policy = retry_policy.clone();
                let channel = channel.clone();
                async move {
                    match delivery {
                        Ok(delivery) => {
                            let routing_keys = RoutingKeys::from(&queue.routing_key);

                            let acknowledger = RabbitMQAcknowledger {
                                delivery: Arc::new(delivery),
                                channel: channel.clone(),
                                routing_keys,
                                retry_policy,
                            };

                            if let Some(policy) = &acknowledger.retry_policy {
                                let attempts = Self::get_retry_attempts(&acknowledger.delivery);
                                if attempts >= policy.max_attempts {
                                    acknowledger.reject(Self::get_error_history(&acknowledger.delivery).join("\n").as_str()).await;
                                    return None;
                                }
                            }

                            let envelope: MessageBrokerEnvelope<Vec<u8>> = match serde_json::from_slice(&acknowledger.delivery.data) {
                                Ok(MessageBrokerEnvelope { name, payload }) => match serde_json::to_vec(&payload as &serde_json::Value) {
                                    Ok(payload) => MessageBrokerEnvelope { name, payload },
                                    Err(e) => {
                                        tracing::error!(error = %e, "Failed to serialize payload");
                                        acknowledger.reject(e.to_string().as_str()).await;
                                        return None;
                                    }
                                },
                                Err(e) => {
                                    tracing::error!(error = %e, "Failed to deserialize envelope");
                                    acknowledger.reject(e.to_string().as_str()).await;
                                    return None;
                                }
                            };

                            Some(BrokerDelivery::new(envelope, Box::new(acknowledger)))
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Delivery error");
                            None
                        }
                    }
                }
            }
        });

        Ok(Box::pin(mapped))
    }

    async fn setup(
        &self,
        queues: Vec<Arc<MessageBrokerQueue>>,
    ) -> Result<(), MessageBrokerSetupError> {
        for queue in queues {
            self.setup_queue(EXCHANGE_NAME, &queue)
                .await
                .map_err(|e| MessageBrokerSetupError::SetupError(e.to_string()))?
        }

        Ok(())
    }
}
