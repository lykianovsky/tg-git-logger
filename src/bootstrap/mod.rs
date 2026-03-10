pub mod executors;
pub mod queues;
pub mod registry;
pub mod workers;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::queues::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::ApplicationBoostrapWorkers;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::DeliveryBotMessengerTelegram;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::listeners::DeliveryEventListeners;
use crate::delivery::http::axum::DeliveryHttpServerAxum;
use crate::delivery::jobs::consumers::move_task_to_test::consumer::MoveTaskToTestJobConsumer;
use crate::delivery::jobs::consumers::send_social_notify::consumer::SendSocialNotifyJobConsumer;
use crate::infrastructure::database::mysql::MySQLDatabase;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::infrastructure::drivers::message_broker::contracts::queue_builder::MessageBrokerQueuesBuilder;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::MessageBrokerRabbitMQ;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::MessageBrokerRabbitMQPublisher;
use crate::infrastructure::processing::event_bus::EventBus;
use serde::Serialize;
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

        let message_broker = Arc::new(
            MessageBrokerRabbitMQ::new(&config.rabbit_mq.url.clone())
                .await
                .expect("Failed to connect to RabbitMQ"),
        );

        let queues = Arc::new(ApplicationQueues::new());

        message_broker
            .setup(
                MessageBrokerQueuesBuilder::new_with_capacity(4)
                    .bind(queues.events.clone())
                    .bind(queues.jobs_critical.clone())
                    .bind(queues.jobs_normal.clone())
                    .bind(queues.jobs_background.clone())
                    .build(),
            )
            .await
            .expect("Failed to setup RabbitMQ scheme");

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

        let job_consumers_registry = Arc::new(
            JobConsumersRegistry::new()
                .register(Arc::new(SendSocialNotifyJobConsumer {
                    executor: executors.commands.send_social_notify.clone(),
                }))
                .await
                .register(Arc::new(MoveTaskToTestJobConsumer {
                    executor: executors.commands.move_task_to_test.clone(),
                }))
                .await,
        );

        let http_server_delivery = DeliveryHttpServerAxum::new(executors.clone(), config.clone());

        let http_server_handle = tokio::spawn(async move {
            http_server_delivery.serve().await.ok();
        });

        let bot_delivery = DeliveryBotMessengerTelegram::new(executors.clone(), config.clone());

        let bot_handle = tokio::spawn(async move {
            bot_delivery.serve().await.ok();
        });

        let event_listeners_delivery = DeliveryEventListeners::new(
            event_bus.clone(),
            publisher.clone(),
            executors.clone(),
            config.clone(),
        );

        let event_listeners_handle = tokio::spawn(async move {
            event_listeners_delivery.serve().await.ok();
        });

        let workers = ApplicationBoostrapWorkers::new(queues.clone(), message_broker.clone());

        workers.run_events(event_bus.clone()).await.ok();
        workers.run_jobs(job_consumers_registry.clone()).await.ok();

        // TODO: Handle shutdown signals and gracefully stop the servers
        tokio::try_join!(http_server_handle, bot_handle, event_listeners_handle).unwrap();
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
