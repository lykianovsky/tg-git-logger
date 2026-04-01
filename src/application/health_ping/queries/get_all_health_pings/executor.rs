use crate::application::health_ping::queries::get_all_health_pings::error::GetAllHealthPingsError;
use crate::application::health_ping::queries::get_all_health_pings::query::GetAllHealthPingsQuery;
use crate::application::health_ping::queries::get_all_health_pings::response::GetAllHealthPingsResponse;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct GetAllHealthPingsExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
}

impl GetAllHealthPingsExecutor {
    pub fn new(health_ping_repo: Arc<dyn HealthPingRepository>) -> Self {
        Self { health_ping_repo }
    }
}

impl CommandExecutor for GetAllHealthPingsExecutor {
    type Command = GetAllHealthPingsQuery;
    type Response = GetAllHealthPingsResponse;
    type Error = GetAllHealthPingsError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let pings = self.health_ping_repo.find_all().await?;

        Ok(GetAllHealthPingsResponse { pings })
    }
}
