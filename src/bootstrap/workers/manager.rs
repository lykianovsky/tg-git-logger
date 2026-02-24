use crate::infrastructure::processing::worker::MessageBrokerWorker;
use tokio::task::JoinSet;

pub struct MessageBrokerWorkerManager<WorkerType, Factory>
where
    Factory: Fn(String) -> WorkerType,
{
    workers: Vec<WorkerType>,
    factory: Factory,
}

impl<WorkerType, Factory> MessageBrokerWorkerManager<WorkerType, Factory>
where
    WorkerType: MessageBrokerWorker + Send + 'static,
    Factory: Fn(String) -> WorkerType + Send + 'static,
{
    pub fn new(factory: Factory) -> Self {
        Self {
            workers: vec![],
            factory,
        }
    }

    pub async fn run(mut self, initial_count: usize) {
        let mut set = JoinSet::new();

        for i in 0..initial_count {
            let worker = (self.factory)(format!("worker_{}", i));
            self.workers.push(worker);
        }

        for worker in self.workers {
            set.spawn(async move {
                worker.start().await.ok();
            });
        }

        while let Some(result) = set.join_next().await {
            if let Err(e) = result {
                tracing::error!("Worker panicked: {:?}", e);
            }
        }
    }
}
