use crate::bootstrap::queues::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::autoscaler::{AutoScaler, AutoScalerConfig};
use crate::bootstrap::workers::events_worker::MessageBrokerEventsWorker;
use crate::bootstrap::workers::jobs_worker::MessageBrokerJobsWorker;
use crate::bootstrap::workers::pool::DynamicWorkerPool;
use crate::bootstrap::workers::stats_provider::BootstrapWorkersStatsProvider;
use crate::domain::monitoring::ports::workers_stats_provider::WorkersStatsProvider;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::processing::event_bus::EventBus;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

struct ManagedPool {
    pool: Arc<Mutex<DynamicWorkerPool>>,
    queue_name: String,
    scaler: AutoScalerConfig,
}

pub struct ApplicationBoostrapWorkersManager {
    pools: Vec<ManagedPool>,
    message_broker: Arc<dyn MessageBroker>,
}

impl ApplicationBoostrapWorkersManager {
    pub fn new(
        queues: Arc<ApplicationQueues>,
        message_broker: Arc<dyn MessageBroker>,
        event_bus: Arc<EventBus>,
        job_consumers: Arc<JobConsumersRegistry>,
    ) -> Self {
        let pools = Self::build_pools(&queues, &message_broker, &event_bus, &job_consumers);
        Self {
            pools,
            message_broker,
        }
    }

    /// Returns an `Arc<dyn WorkersStatsProvider>` that can be passed to the application layer.
    pub fn stats_provider(&self) -> Arc<dyn WorkersStatsProvider> {
        let pools = self
            .pools
            .iter()
            .map(|p| (p.queue_name.clone(), p.pool.clone()))
            .collect();

        Arc::new(BootstrapWorkersStatsProvider::new(
            pools,
            self.message_broker.clone(),
        ))
    }

    pub async fn run(self) {
        let mut set = JoinSet::new();

        for managed in self.pools {
            {
                let mut pool = managed.pool.lock().await;
                for _ in 0..managed.scaler.min_workers {
                    pool.spawn_worker();
                }
            }

            let autoscaler = AutoScaler::new(
                managed.queue_name,
                managed.pool,
                self.message_broker.clone(),
                managed.scaler,
            );

            set.spawn(async move { autoscaler.run().await });
        }

        while let Some(r) = set.join_next().await {
            if let Err(e) = r {
                tracing::error!("AutoScaler panicked: {:?}", e);
            }
        }
    }

    fn build_pools(
        queues: &Arc<ApplicationQueues>,
        message_broker: &Arc<dyn MessageBroker>,
        event_bus: &Arc<EventBus>,
        job_consumers: &Arc<JobConsumersRegistry>,
    ) -> Vec<ManagedPool> {
        let mut pools = Vec::new();

        // Events queue
        {
            let queue = queues.events.clone();
            let event_bus = event_bus.clone();
            let broker = message_broker.clone();
            let queue_name = queue.name.clone();

            let pool = DynamicWorkerPool::new(queue_name.clone(), move |name| {
                Box::new(MessageBrokerEventsWorker::new(
                    &name,
                    queue.clone(),
                    event_bus.clone(),
                    broker.clone(),
                ))
            });

            pools.push(ManagedPool {
                pool: Arc::new(Mutex::new(pool)),
                queue_name,
                scaler: AutoScalerConfig {
                    min_workers: 1,
                    max_workers: 3,
                    tasks_per_worker_threshold: 2,
                    idle_ticks_to_scale_down: 3,
                    poll_interval: Duration::from_secs(10),
                },
            });
        }

        // Jobs queues
        let jobs_configs = [
            (
                queues.jobs_critical.clone(),
                AutoScalerConfig {
                    min_workers: 2,
                    max_workers: 10,
                    tasks_per_worker_threshold: 2,
                    idle_ticks_to_scale_down: 5,
                    poll_interval: Duration::from_secs(10),
                },
            ),
            (
                queues.jobs_normal.clone(),
                AutoScalerConfig {
                    min_workers: 1,
                    max_workers: 5,
                    tasks_per_worker_threshold: 3,
                    idle_ticks_to_scale_down: 3,
                    poll_interval: Duration::from_secs(10),
                },
            ),
            (
                queues.jobs_background.clone(),
                AutoScalerConfig {
                    min_workers: 1,
                    max_workers: 3,
                    tasks_per_worker_threshold: 5,
                    idle_ticks_to_scale_down: 2,
                    poll_interval: Duration::from_secs(15),
                },
            ),
        ];

        for (queue, scaler_config) in jobs_configs {
            let consumers = job_consumers.clone();
            let broker = message_broker.clone();
            let queue_name = queue.name.clone();

            let pool = DynamicWorkerPool::new(queue_name.clone(), move |name| {
                Box::new(MessageBrokerJobsWorker::new(
                    &name,
                    queue.clone(),
                    consumers.clone(),
                    broker.clone(),
                ))
            });

            pools.push(ManagedPool {
                pool: Arc::new(Mutex::new(pool)),
                queue_name,
                scaler: scaler_config,
            });
        }

        pools
    }
}
