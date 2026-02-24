pub mod send_email;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

pub struct DeliveryJobConsumers {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryJobConsumers {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        Self { executors, config }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryJobConsumers {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
