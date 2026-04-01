use crate::application::health_ping::commands::update_health_ping_status::command::UpdateHealthPingStatusCommand;
use crate::application::health_ping::commands::update_health_ping_status::error::UpdateHealthPingStatusExecutorError;
use crate::application::health_ping::commands::update_health_ping_status::response::UpdateHealthPingStatusResponse;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct UpdateHealthPingStatusExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
}

impl UpdateHealthPingStatusExecutor {
    pub fn new(health_ping_repo: Arc<dyn HealthPingRepository>) -> Self {
        Self { health_ping_repo }
    }
}

impl CommandExecutor for UpdateHealthPingStatusExecutor {
    type Command = UpdateHealthPingStatusCommand;
    type Response = UpdateHealthPingStatusResponse;
    type Error = UpdateHealthPingStatusExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let mut ping = self.health_ping_repo.find_by_id(cmd.id).await?;

        let previous_status = ping.last_status.clone();

        ping.last_status = Some(cmd.status.clone());
        ping.last_response_ms = cmd.response_ms;
        ping.last_error_message = cmd.error_message.clone();
        ping.last_checked_at = Some(cmd.checked_at);

        self.health_ping_repo.update(&ping).await?;

        Ok(UpdateHealthPingStatusResponse {
            previous_status,
            new_status: cmd.status.clone(),
        })
    }
}
