pub mod executors;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::DeliveryBotMessengerTelegram;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::rabbitmq::DeliveryRabbitMqEvents;
use crate::delivery::http::axum::DeliveryHttpServerAxum;
use crate::domain::shared::events::publisher::EventPublisher;
use crate::infrastructure::database::mysql::MySQLDatabase;
use crate::infrastructure::drivers::message_broker::rabbitmq::connector::RabbitMQConnector;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::RabbitMqPublisher;
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

        let rabbitmq_pool = Arc::new(
            RabbitMQConnector::new(config.rabbit_mq.url.clone())
                .await
                .expect("RabbitMQ connection failed"),
        );

        let event_publisher: Arc<dyn EventPublisher> = Arc::new(
            RabbitMqPublisher::new(rabbitmq_pool.clone())
                .await
                .expect("RabbitMQ event publisher created failed"),
        );

        let executors = Arc::new(ApplicationBoostrapExecutors::new(
            config.clone(),
            mysql_pool.clone(),
            event_publisher.clone(),
        ));

        let http_server_delivery = DeliveryHttpServerAxum::new(executors.clone(), config.clone());

        let http_server_handle = tokio::spawn(async move {
            let _ = http_server_delivery.serve().await;
        });

        let bot_delivery = DeliveryBotMessengerTelegram::new(executors.clone(), config.clone());

        let bot_handle = tokio::spawn(async move {
            let _ = bot_delivery.serve().await;
        });

        let events_delivery =
            DeliveryRabbitMqEvents::new(executors.clone(), config.clone(), rabbitmq_pool.clone());

        let events_handle = tokio::spawn(async move {
            let _ = events_delivery.serve().await;
        });

        tokio::try_join!(http_server_handle, bot_handle, events_handle).unwrap();
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
