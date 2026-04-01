use crate::domain::monitoring::ports::workers_stats_provider::QueueWorkerStats;

pub struct GetQueuesStatsResponse {
    pub stats: Vec<QueueWorkerStats>,
}
