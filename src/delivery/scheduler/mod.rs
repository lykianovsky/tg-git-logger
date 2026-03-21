use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct DeliveryScheduler {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryScheduler {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        Self { executors, config }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryScheduler {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let scheduler = JobScheduler::new()
            .await
            .expect("JobScheduler not initialized");

        scheduler
            .add(
                Job::new_async("0/10 * * * * *", |_uuid, _lock| {
                    Box::pin(async {
                        println!("Выполняю джобу каждые 10 секунд");
                    })
                })
                .expect("Async job create error"),
            )
            .await
            .expect("JobScheduler failed");

        Ok(())
    }
}
