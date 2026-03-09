use crate::bootstrap::queue::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::events_worker::MessageBrokerEventsWorker;
use crate::bootstrap::workers::jobs_worker::MessageBrokerJobsWorker;
use crate::delivery::jobs::consumers::send_email::consumer::SendEmailJobConsumer;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::processing::event_bus::EventBus;
use std::sync::Arc;

mod events_worker;
mod jobs_worker;
mod manager;

pub struct ApplicationBoostrapWorkers {
    queues: Arc<ApplicationQueues>,
    event_bus: Arc<EventBus>,
    message_broker: Arc<dyn MessageBroker>,
}

impl ApplicationBoostrapWorkers {
    pub fn new(
        queues: Arc<ApplicationQueues>,
        event_bus: Arc<EventBus>,
        message_broker: Arc<dyn MessageBroker>,
    ) -> Self {
        Self {
            queues,
            event_bus,
            message_broker,
        }
    }

    pub async fn run(&self) -> Result<(), String> {
        let queue = self.queues.events.clone();
        let event_bus = self.event_bus.clone();
        let message_broker = self.message_broker.clone();

        tokio::spawn(async move {
            manager::MessageBrokerWorkerManager::new(move |name| {
                MessageBrokerEventsWorker::new(
                    name.as_str(),
                    queue.clone(),
                    event_bus.clone(),
                    message_broker.clone(),
                )
            })
            .run(1)
            .await;
        });

        let queue = self.queues.jobs.clone();
        let message_broker = self.message_broker.clone();

        let job_consumers_registry = Arc::new(
            JobConsumersRegistry::new()
                .register(Arc::new(SendEmailJobConsumer {}))
                .await,
        );

        tokio::spawn(async move {
            manager::MessageBrokerWorkerManager::new(move |name| {
                MessageBrokerJobsWorker::new(
                    name.as_str(),
                    queue.clone(),
                    job_consumers_registry.clone(),
                    message_broker.clone(),
                )
            })
            .run(1)
            .await;
        });

        Ok(())
    }
}
