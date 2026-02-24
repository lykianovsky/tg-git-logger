use crate::infrastructure::processing::job::JobConsumer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct JobConsumersRegistry {
    jobs: RwLock<HashMap<String, Arc<dyn JobConsumer>>>,
}

impl JobConsumersRegistry {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(self, job: Arc<dyn JobConsumer>) -> Self {
        self.jobs.write().await.insert(job.name().to_string(), job);

        self
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn JobConsumer>> {
        self.jobs.read().await.get(name).cloned()
    }
}
