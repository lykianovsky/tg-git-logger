#[async_trait::async_trait]
pub trait CacheService: Send + Sync {
    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String>;

    async fn get(&self, key: &str) -> Result<Option<String>, String>;

    async fn del(&self, key: &str) -> Result<(), String>;

    async fn take(&self, key: &str) -> Result<Option<String>, String>;
}
