use crate::infrastructure::drivers::cache::contract::CacheService;
use hex::encode;
use hmac::{Hmac, Mac};
use redis::AsyncCommands;
use serde::Serialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(url: String) -> Self {
        let client = redis::Client::open(url).unwrap();
        Self { client }
    }

    fn make_key_by_payload<T: Serialize>(&self, secret: &str, payload: &T) -> String {
        let json = serde_json::to_string(payload).expect("Failed to serialize payload");

        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");

        mac.update(json.as_bytes());

        encode(mac.finalize().into_bytes())
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
