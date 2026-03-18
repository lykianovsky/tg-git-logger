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

        tracing::info!("Starting MessageBrokerWorkerManager");
        tracing::info!("Spawning {} workers", initial_count);

        for i in 0..initial_count {
            let worker_name = format!("worker_{}", i);
            let worker = (self.factory)(worker_name.clone());
            self.workers.push(worker);
            tracing::info!("Spawn {} worker", worker_name);
        }

        tracing::info!("All {} workers created successfully", self.workers.len());

        for worker in self.workers {
            set.spawn(async move {
                let worker_name = worker.name().to_string();
                match worker.start().await {
                    Ok(_) => {
                        tracing::info!("Worker {} completed execution successfully", worker_name);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Worker {} completed execution with error: {}",
                            worker_name,
                            e.to_string()
                        );
                    }
                };
            });
        }

        tracing::info!("All workers started, listening for messages...");

        while let Some(result) = set.join_next().await {
            if let Err(e) = result {
                tracing::error!("Worker panicked: {:?}", e);
            }
        }
    }
}
