use crate::application::monitoring::queries::get_queues_stats::error::GetQueuesStatsError;
use crate::application::monitoring::queries::get_queues_stats::query::GetQueuesStatsQuery;
use crate::application::monitoring::queries::get_queues_stats::response::GetQueuesStatsResponse;
use crate::domain::monitoring::ports::workers_stats_provider::WorkersStatsProvider;
use crate::domain::shared::command::CommandExecutor;
use std::sync::Arc;

pub struct GetQueuesStatsExecutor {
    pub stats_provider: Arc<dyn WorkersStatsProvider>,
}

impl CommandExecutor for GetQueuesStatsExecutor {
    type Command = GetQueuesStatsQuery;
    type Response = GetQueuesStatsResponse;
    type Error = GetQueuesStatsError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let stats = self.stats_provider.get_stats().await;
        Ok(GetQueuesStatsResponse { stats })
    }
}
