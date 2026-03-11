use crate::bootstrap::queues::ApplicationQueues;
use crate::bootstrap::registry::jobs::JobConsumersRegistry;
use crate::bootstrap::workers::events_worker::MessageBrokerEventsWorker;
use crate::bootstrap::workers::jobs_worker::MessageBrokerJobsWorker;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::processing::event_bus::EventBus;
use std::sync::Arc;

mod events_worker;
mod jobs_worker;
mod manager;

pub struct ApplicationBoostrapWorkers {
    queues: Arc<ApplicationQueues>,
    message_broker: Arc<dyn MessageBroker>,
}

impl ApplicationBoostrapWorkers {
    pub fn new(queues: Arc<ApplicationQueues>, message_broker: Arc<dyn MessageBroker>) -> Self {
        Self {
            queues,
            message_broker,
        }
    }

    pub async fn run_events(&self, event_bus: Arc<EventBus>) -> Result<(), String> {
        let queue = self.queues.events.clone();
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

        Ok(())
    }

    pub async fn run_jobs(&self, job_consumers: Arc<JobConsumersRegistry>) -> Result<(), String> {
        let queue_critical = self.queues.jobs_critical.clone();
        let job_consumers_clone = job_consumers.clone();
        let message_broker_clone = self.message_broker.clone();

        tokio::spawn(async move {
            manager::MessageBrokerWorkerManager::new(move |name| {
                MessageBrokerJobsWorker::new(
                    name.as_str(),
                    queue_critical.clone(),
                    job_consumers_clone.clone(),
                    message_broker_clone.clone(),
                )
            })
            .run(1)
            .await;
        });

        let job_consumers_clone = job_consumers.clone();
        let message_broker_clone = self.message_broker.clone();
        let queue_normal = self.queues.jobs_normal.clone();

        tokio::spawn(async move {
            manager::MessageBrokerWorkerManager::new(move |name| {
                MessageBrokerJobsWorker::new(
                    name.as_str(),
                    queue_normal.clone(),
                    job_consumers_clone.clone(),
                    message_broker_clone.clone(),
                )
            })
            .run(1)
            .await;
        });

        let job_consumers_clone = job_consumers.clone();
        let message_broker_clone = self.message_broker.clone();
        let queue_background = self.queues.jobs_background.clone();

        tokio::spawn(async move {
            manager::MessageBrokerWorkerManager::new(move |name| {
                MessageBrokerJobsWorker::new(
                    name.as_str(),
                    queue_background.clone(),
                    job_consumers_clone.clone(),
                    message_broker_clone.clone(),
                )
            })
            .run(1)
            .await;
        });

        Ok(())
    }
}
