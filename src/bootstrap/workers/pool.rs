use crate::infrastructure::processing::worker::MessageBrokerWorker;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

type WorkerFactory = Box<dyn Fn(String) -> Box<dyn MessageBrokerWorker> + Send + Sync + 'static>;

struct WorkerHandle {
    name: String,
    cancel: CancellationToken,
    join: JoinHandle<()>,
}

pub struct DynamicWorkerPool {
    name: String,
    counter: usize,
    handles: Vec<WorkerHandle>,
    factory: WorkerFactory,
}

impl DynamicWorkerPool {
    pub fn new<F>(name: String, factory: F) -> Self
    where
        F: Fn(String) -> Box<dyn MessageBrokerWorker> + Send + Sync + 'static,
    {
        Self {
            name,
            counter: 0,
            handles: vec![],
            factory: Box::new(factory),
        }
    }

    pub fn spawn_worker(&mut self) {
        let worker_name = format!("worker:{}:{}", self.name, self.counter);
        self.counter += 1;

        let worker = (self.factory)(worker_name.clone());
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let name_clone = worker_name.clone();

        let join = tokio::spawn(async move {
            match worker.start(cancel_clone).await {
                Ok(_) => tracing::info!(worker = %name_clone, "Worker finished"),
                Err(e) => tracing::error!(worker = %name_clone, error = %e, "Worker error"),
            }
        });

        self.handles.push(WorkerHandle {
            name: worker_name,
            cancel,
            join,
        });
    }

    /// Cancels and removes the most recently spawned worker.
    pub fn stop_one(&mut self) {
        if let Some(handle) = self.handles.pop() {
            tracing::info!(worker = %handle.name, "Scaling down: cancelling worker");
            handle.cancel.cancel();
        }
    }

    pub fn active_count(&self) -> usize {
        self.handles.len()
    }

    /// Removes handles of workers that finished on their own (stream closed / error),
    /// returns the number removed so the caller can respawn if needed.
    pub fn collect_dead(&mut self) -> usize {
        let before = self.handles.len();
        self.handles.retain(|h| !h.join.is_finished());
        let dead = before - self.handles.len();

        if dead > 0 {
            tracing::warn!(pool = %self.name, count = dead, "Collected dead workers");
        }

        dead
    }

    /// Restarts workers that died unexpectedly.
    pub fn restart_dead(&mut self) {
        let dead = self.collect_dead();
        for _ in 0..dead {
            tracing::warn!(pool = %self.name, "Restarting dead worker");
            self.spawn_worker();
        }
    }
}
