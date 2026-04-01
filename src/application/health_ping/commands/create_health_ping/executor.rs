use crate::application::health_ping::commands::create_health_ping::command::CreateHealthPingCommand;
use crate::application::health_ping::commands::create_health_ping::error::CreateHealthPingExecutorError;
use crate::application::health_ping::commands::create_health_ping::response::CreateHealthPingResponse;
use crate::domain::health_ping::entities::health_ping::HealthPing;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use crate::domain::shared::command::CommandExecutor;
use chrono::Utc;
use std::sync::Arc;

pub struct CreateHealthPingExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
}

impl CreateHealthPingExecutor {
    pub fn new(health_ping_repo: Arc<dyn HealthPingRepository>) -> Self {
        Self { health_ping_repo }
    }
}

impl CommandExecutor for CreateHealthPingExecutor {
    type Command = CreateHealthPingCommand;
    type Response = CreateHealthPingResponse;
    type Error = CreateHealthPingExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let ping = HealthPing {
            id: HealthPingId::default(),
            name: cmd.name.clone(),
            url: cmd.url.clone(),
            interval_minutes: cmd.interval_minutes,
            is_active: true,
            last_checked_at: None,
            last_status: None,
            last_response_ms: None,
            last_error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = self.health_ping_repo.create(&ping).await?;

        Ok(CreateHealthPingResponse { ping: created })
    }
}
