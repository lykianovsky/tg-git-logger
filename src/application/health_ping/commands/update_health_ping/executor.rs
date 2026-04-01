use crate::application::health_ping::commands::update_health_ping::command::UpdateHealthPingCommand;
use crate::application::health_ping::commands::update_health_ping::error::UpdateHealthPingExecutorError;
use crate::application::health_ping::commands::update_health_ping::response::UpdateHealthPingResponse;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct UpdateHealthPingExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
}

impl UpdateHealthPingExecutor {
    pub fn new(health_ping_repo: Arc<dyn HealthPingRepository>) -> Self {
        Self { health_ping_repo }
    }
}

impl CommandExecutor for UpdateHealthPingExecutor {
    type Command = UpdateHealthPingCommand;
    type Response = UpdateHealthPingResponse;
    type Error = UpdateHealthPingExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut ping = self.health_ping_repo.find_by_id(cmd.id).await?;

        if let Some(ref name) = cmd.name {
            ping.name = name.clone();
        }

        if let Some(ref url) = cmd.url {
            ping.url = url.clone();
        }

        if let Some(interval) = cmd.interval_minutes {
            ping.interval_minutes = interval;
        }

        if let Some(is_active) = cmd.is_active {
            ping.is_active = is_active;
        }

        self.health_ping_repo.update(&ping).await?;

        Ok(UpdateHealthPingResponse)
    }
}
