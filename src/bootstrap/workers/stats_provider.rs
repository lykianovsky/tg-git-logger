use crate::bootstrap::workers::pool::DynamicWorkerPool;
use crate::domain::monitoring::ports::workers_stats_provider::{
    QueueWorkerStats, WorkersStatsProvider,
};
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BootstrapWorkersStatsProvider {
    pools: Vec<(String, Arc<Mutex<DynamicWorkerPool>>)>,
    broker: Arc<dyn MessageBroker>,
}

impl BootstrapWorkersStatsProvider {
    pub fn new(
        pools: Vec<(String, Arc<Mutex<DynamicWorkerPool>>)>,
        broker: Arc<dyn MessageBroker>,
    ) -> Self {
        Self { pools, broker }
    }
}

#[async_trait]
impl WorkersStatsProvider for BootstrapWorkersStatsProvider {
    async fn get_stats(&self) -> Vec<QueueWorkerStats> {
        let mut result = Vec::with_capacity(self.pools.len());

        for (queue_name, pool) in &self.pools {
            let active_workers = pool.lock().await.active_count();

            let pending_messages = self.broker.queue_depth(queue_name).await.unwrap_or(0);

            result.push(QueueWorkerStats {
                queue_name: queue_name.clone(),
                active_workers,
                pending_messages,
            });
        }

        result
    }
}
