use crate::infrastructure::processing::worker::MessageBrokerWorker;
use tokio::task::JoinSet;

type WorkerFactory = Box<dyn Fn(String) -> Box<dyn MessageBrokerWorker> + Send + 'static>;

pub struct MessageBrokerWorkerPool {
    name: String,
    workers: Vec<Box<dyn MessageBrokerWorker>>,
    factory: WorkerFactory,
}

impl MessageBrokerWorkerPool {
    pub fn new<F>(name: String, factory: F) -> Self
    where
        F: Fn(String) -> Box<dyn MessageBrokerWorker> + Send + 'static,
    {
        Self {
            name,
            workers: vec![],
            factory: Box::new(factory),
        }
    }

    pub async fn run(mut self, initial_count: usize) {
        let mut set = JoinSet::new();

        for i in 0..initial_count {
            let worker_name = format!("worker:{}:{}", self.name, i);
            let worker = (self.factory)(worker_name.clone());
            self.workers.push(worker);
            tracing::info!("Spawned worker {}", worker_name);
        }

        for worker in self.workers {
            set.spawn(async move {
                let name = worker.name().to_string();
                match worker.start().await {
                    Ok(_) => tracing::info!("Worker {} done", name),
                    Err(e) => tracing::error!("Worker {} error: {}", name, e),
                }
            });
        }

        while let Some(result) = set.join_next().await {
            if let Err(e) = result {
                tracing::error!("Worker panicked: {:?}", e);
            }
        }
    }
}
