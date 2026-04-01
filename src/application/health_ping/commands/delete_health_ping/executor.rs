use crate::application::health_ping::commands::delete_health_ping::command::DeleteHealthPingCommand;
use crate::application::health_ping::commands::delete_health_ping::error::DeleteHealthPingExecutorError;
use crate::application::health_ping::commands::delete_health_ping::response::DeleteHealthPingResponse;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct DeleteHealthPingExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
}

impl DeleteHealthPingExecutor {
    pub fn new(health_ping_repo: Arc<dyn HealthPingRepository>) -> Self {
        Self { health_ping_repo }
    }
}

impl CommandExecutor for DeleteHealthPingExecutor {
    type Command = DeleteHealthPingCommand;
    type Response = DeleteHealthPingResponse;
    type Error = DeleteHealthPingExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        self.health_ping_repo.delete(cmd.id).await?;

        Ok(DeleteHealthPingResponse)
    }
}
