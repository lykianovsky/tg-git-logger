use crate::config::environment::ENV;
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
    pub fn new() -> Self {
        let redis_url = ENV.get("REDIS_URL");
        let client = redis::Client::open(redis_url).unwrap();
        Self { client }
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> redis::RedisResult<()> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.set_ex(key, value, ttl_secs).await
    }

    pub async fn get(&self, key: &str) -> redis::RedisResult<Option<String>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.get(key).await
    }

    pub async fn del(&self, key: &str) -> redis::RedisResult<()> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.del(key).await
    }

    pub async fn take(&self, key: &str) -> redis::RedisResult<Option<String>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;

        redis::cmd("GETDEL").arg(key).query_async(&mut con).await
    }

    pub fn make_key_by_payload<T: Serialize>(&self, payload: &T) -> String {
        let json = serde_json::to_string(payload).expect("Failed to serialize payload");

        let mut mac = HmacSha256::new_from_slice(ENV.get("REDIS_SECRET_KEY").as_bytes())
            .expect("HMAC can take key of any size");

        mac.update(json.as_bytes());

        encode(mac.finalize().into_bytes())
    }
}
