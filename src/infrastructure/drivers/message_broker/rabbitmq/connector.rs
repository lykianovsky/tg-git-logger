use lapin::{Connection, ConnectionProperties};
use std::sync::Arc;

pub struct RabbitMQConnector {
    inner: Arc<Connection>,
}

impl RabbitMQConnector {
    pub async fn new(url: String) -> Result<Self, lapin::Error> {
        let connect = Connection::connect(&url, ConnectionProperties::default()).await?;

        Ok(Self {
            inner: Arc::new(connect),
        })
    }

    pub async fn create_channel(&self) -> Result<lapin::Channel, lapin::Error> {
        self.inner.create_channel().await
    }
}
