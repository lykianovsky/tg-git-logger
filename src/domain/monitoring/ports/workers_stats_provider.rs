use async_trait::async_trait;

pub struct QueueWorkerStats {
    pub queue_name: String,
    pub active_workers: usize,
    pub pending_messages: u32,
}

#[async_trait]
pub trait WorkersStatsProvider: Send + Sync {
    async fn get_stats(&self) -> Vec<QueueWorkerStats>;
}
