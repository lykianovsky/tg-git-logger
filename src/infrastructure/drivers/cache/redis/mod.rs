use crate::infrastructure::drivers::cache::contract::CacheService;
use redis::AsyncCommands;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(url: String) -> Self {
        let client = redis::Client::open(url).unwrap();
        Self { client }
    }
}

#[async_trait::async_trait]
impl CacheService for RedisCache {
    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        // тут не присваиваем
        let result: Option<()> = con
            .set_ex(key, value, ttl_secs)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<String>, String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        let result = con.get(key).await.map_err(|e| e.to_string())?;

        Ok(result)
    }

    async fn del(&self, key: &str) -> Result<(), String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        con.del(key).await.map_err(|e| e.to_string())
    }

    async fn take(&self, key: &str) -> Result<Option<String>, String> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| e.to_string())?;

        let result: Option<String> = redis::cmd("GETDEL")
            .arg(key)
            .query_async(&mut con)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}
