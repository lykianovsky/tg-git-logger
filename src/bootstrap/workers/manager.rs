use crate::bootstrap::queues::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::events_worker::MessageBrokerEventsWorker;
use crate::bootstrap::workers::jobs_worker::MessageBrokerJobsWorker;
use crate::bootstrap::workers::pool::MessageBrokerWorkerPool;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::processing::event_bus::EventBus;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

pub struct ApplicationBoostrapWorkersManager {
    queues: Arc<ApplicationQueues>,
    event_bus: Arc<EventBus>,
    message_broker: Arc<dyn MessageBroker>,
    job_consumers: Arc<JobConsumersRegistry>,
    pools: Mutex<HashMap<String, MessageBrokerWorkerPool>>,
}

impl ApplicationBoostrapWorkersManager {
    pub fn new(
        queues: Arc<ApplicationQueues>,
        message_broker: Arc<dyn MessageBroker>,
        event_bus: Arc<EventBus>,
        job_consumers: Arc<JobConsumersRegistry>,
    ) -> Self {
        Self {
            event_bus,
            queues,
            message_broker,
            job_consumers,
            pools: Mutex::new(HashMap::new()),
        }
    }

    pub async fn run(self) {
        self.register_jobs().await;
        self.register_events().await;

        let mut set = JoinSet::new();

        for (name, pool) in self.pools.into_inner() {
            tracing::debug!("Start pool: {name}");

            set.spawn(async move { pool.run(5).await });
        }

        while let Some(r) = set.join_next().await {
            if let Err(e) = r {
                tracing::error!("Pool panicked: {:?}", e);
            }
        }
    }

    async fn add_pool(&self, name: &str, pool: MessageBrokerWorkerPool) {
        let mut map = self.pools.lock().await;

        map.insert(name.to_string(), pool);
    }

    async fn register_events(&self) {
        tracing::debug!("Register workers for events queue");

        let queue = self.queues.events.clone();
        let message_broker = self.message_broker.clone();
        let event_bus = self.event_bus.clone();
        let queue_name = &self.queues.events.name;

        let pool = MessageBrokerWorkerPool::new(queue_name.clone(), move |name| {
            Box::new(MessageBrokerEventsWorker::new(
                name.as_str(),
                queue.clone(),
                event_bus.clone(),
                message_broker.clone(),
            ))
        });

        self.add_pool(queue_name, pool).await;

        tracing::debug!("Successfully register workers for events queue");
    }

    pub async fn register_jobs(&self) {
        tracing::debug!("Register workers for jobs queue");

        for queue in [
            self.queues.jobs_critical.clone(),
            self.queues.jobs_normal.clone(),
            self.queues.jobs_background.clone(),
        ] {
            let consumers = self.job_consumers.clone();
            let broker = self.message_broker.clone();
            let queue_name = queue.name.clone();

            let pool = MessageBrokerWorkerPool::new(queue_name.clone(), move |name| {
                Box::new(MessageBrokerJobsWorker::new(
                    name.as_str(),
                    queue.clone(),
                    consumers.clone(),
                    broker.clone(),
                ))
            });

            self.add_pool(&queue_name, pool).await;
        }

        tracing::debug!("Successfully register workers for jobs queue");
    }
}
