use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RabbitMQMessageBrokerRetryError {
    pub reason: String,
    pub at: DateTime<Utc>,
}
