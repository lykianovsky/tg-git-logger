pub mod executors;
pub mod queue;
pub mod registry;
pub mod workers;

#[derive(Deserialize, Serialize)]
pub struct TestEvent {
    pub(crate) keys: String,
}

impl DomainEvent for TestEvent {
    const EVENT_NAME: &'static str = "test.event";
}

impl MessageBrokerMessage for TestEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}

pub struct TestLister {}

#[async_trait]
impl EventListener<TestEvent> for TestLister {
    async fn handle(&self, payload: &TestEvent) {
        tracing::debug!("TEST PAYLOAD ASD ASDASDASDASDASD  {:?}", payload.keys);
    }
}

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::queue::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::ApplicationBoostrapWorkers;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::DeliveryBotMessengerTelegram;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::http::axum::DeliveryHttpServerAxum;
use crate::domain::shared::events::event::DomainEvent;
use crate::domain::shared::events::event_listener::EventListener;
use crate::infrastructure::database::mysql::MySQLDatabase;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind, MessageBrokerPublisher,
};
use crate::infrastructure::drivers::message_broker::contracts::queue::{
    MessageBrokerQueue, MessageBrokerQueueRetryPolicy,
};
use crate::infrastructure::drivers::message_broker::contracts::queue_builder::MessageBrokerQueuesBuilder;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::MessageBrokerRabbitMQ;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::MessageBrokerRabbitMQPublisher;
use crate::infrastructure::processing::event_bus::EventBus;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct ApplicationBootstrap;

impl ApplicationBootstrap {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self) {
        let config = Arc::new(ApplicationConfig::new());

        self.setup_logging(&config);

        let mysql_pool = Arc::new(MySQLDatabase::new(config.mysql.url.clone()).connect().await);

        let event_bus = Arc::new(EventBus::new());

        tracing::debug!(
            "{:?}",
            serde_json::to_vec(&TestEvent {
                keys: "".to_string(),
            })
            .unwrap()
        );
        event_bus.on(TestLister {}).await;

        let message_broker = Arc::new(
            MessageBrokerRabbitMQ::new(&config.rabbit_mq.url.clone())
                .await
                .expect("Failed to connect to RabbitMQ"),
        );

        let queues = Arc::new(ApplicationQueues::new());

        message_broker
            .setup(
                MessageBrokerQueuesBuilder::new_with_capacity(2)
                    .bind(queues.events.clone())
                    .bind(queues.jobs.clone())
                    .build(),
            )
            .await
            .expect("Failed to setup RabbitMQ s2cheme");

        let publisher: Arc<dyn MessageBrokerPublisher> = Arc::new(
            MessageBrokerRabbitMQPublisher::new(message_broker.connection.clone())
                .await
                .unwrap(),
        );

        let executors = Arc::new(ApplicationBoostrapExecutors::new(
            config.clone(),
            mysql_pool.clone(),
            publisher.clone(),
        ));

        let http_server_delivery = DeliveryHttpServerAxum::new(executors.clone(), config.clone());

        let http_server_handle = tokio::spawn(async move {
            http_server_delivery.serve().await.ok();
        });

        let bot_delivery = DeliveryBotMessengerTelegram::new(executors.clone(), config.clone());

        let bot_handle = tokio::spawn(async move {
            bot_delivery.serve().await.ok();
        });

        ApplicationBoostrapWorkers::new(queues.clone(), event_bus.clone(), message_broker.clone())
            .run()
            .await
            .ok();

        // TODO: Handle shutdown signals and gracefully stop the servers
        tokio::try_join!(http_server_handle, bot_handle).unwrap();
    }

    fn setup_logging(&self, config: &ApplicationConfig) {
        let debug_filter: &str = if config.debug { "debug" } else { "info" };

        tracing_subscriber::fmt()
            .with_env_filter(debug_filter)
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::ENTER
                    | tracing_subscriber::fmt::format::FmtSpan::EXIT,
            )
            .with_target(true)
            .with_file(true)
            .with_line_number(config.debug)
            .init();
    }
}
