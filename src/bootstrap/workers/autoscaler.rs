use crate::bootstrap::workers::pool::DynamicWorkerPool;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct AutoScalerConfig {
    /// Minimum number of workers — never scale below this.
    pub min_workers: usize,
    /// Maximum number of workers — never scale above this.
    pub max_workers: usize,
    /// Max tasks per worker before spawning a new one.
    /// Scale up when `depth > active_workers * tasks_per_worker_threshold`.
    pub tasks_per_worker_threshold: usize,
    /// How many consecutive idle ticks before scaling down by one.
    pub idle_ticks_to_scale_down: u32,
    /// How often to check queue depth.
    pub poll_interval: Duration,
}

pub struct AutoScaler {
    queue_name: String,
    pool: Arc<Mutex<DynamicWorkerPool>>,
    broker: Arc<dyn MessageBroker>,
    config: AutoScalerConfig,
    idle_ticks: u32,
}

impl AutoScaler {
    pub fn new(
        queue_name: String,
        pool: Arc<Mutex<DynamicWorkerPool>>,
        broker: Arc<dyn MessageBroker>,
        config: AutoScalerConfig,
    ) -> Self {
        Self {
            queue_name,
            pool,
            broker,
            config,
            idle_ticks: 0,
        }
    }

    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(self.config.poll_interval);

        loop {
            interval.tick().await;
            self.tick().await;
        }
    }

    async fn tick(&mut self) {
        let mut pool = self.pool.lock().await;

        pool.restart_dead();

        let depth = match self.broker.queue_depth(&self.queue_name).await {
            Ok(d) => d as usize,
            Err(e) => {
                tracing::warn!(
                    queue = %self.queue_name,
                    error = %e,
                    "AutoScaler: failed to get queue depth, skipping tick"
                );
                return;
            }
        };

        let active = pool.active_count();

        tracing::debug!(
            queue = %self.queue_name,
            depth = depth,
            workers = active,
            idle_ticks = self.idle_ticks,
            "AutoScaler tick"
        );

        if depth == 0 {
            self.idle_ticks += 1;

            if self.idle_ticks >= self.config.idle_ticks_to_scale_down
                && active > self.config.min_workers
            {
                pool.stop_one();
                self.idle_ticks = 0;

                tracing::info!(
                    queue = %self.queue_name,
                    workers = active - 1,
                    "AutoScaler: scaled down (idle)"
                );
            }
        } else {
            self.idle_ticks = 0;

            if depth > active * self.config.tasks_per_worker_threshold
                && active < self.config.max_workers
            {
                pool.spawn_worker();

                tracing::info!(
                    queue = %self.queue_name,
                    depth = depth,
                    workers = active + 1,
                    "AutoScaler: scaled up"
                );
            }
        }
    }
}
