use crate::builder::RNRedisBuilder;
use crate::error::RedisError;
use fred::interfaces::{KeysInterface, LuaInterface};
use fred::types::{Expiration, FromValue};
use serde::{de::DeserializeOwned, Serialize};

/// Redis client wrapper providing cluster-native high-level operations.
#[derive(Clone)]
pub struct RNRedis {
    client: fred::clients::Client,
}

impl RNRedis {
    /// Creates a new RNRedis instance wrapping a fred Client.
    pub fn from_client(client: fred::clients::Client) -> Self {
        Self { client }
    }

    /// Returns a reference to the underlying fred Client.
    pub fn client(&self) -> &fred::clients::Client {
        &self.client
    }

    /// Creates a new builder instance.
    pub fn builder() -> RNRedisBuilder {
        RNRedisBuilder::new()
    }

    /// Connects to a Redis cluster node URL with default settings.
    pub async fn connect(url: &str) -> Result<Self, RedisError> {
        Self::builder().add_node(Some(url)).build().await
    }

    /// Retrieves an item from Redis and deserializes it from JSON.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let raw: Option<String> = self.client.get(key).await?;
        match raw {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    /// Retrieves a raw string from Redis.
    pub async fn get_str(&self, key: &str) -> Result<Option<String>, RedisError> {
        let raw: Option<String> = self.client.get(key).await?;
        Ok(raw)
    }

    /// Serializes an item to JSON and stores it in Redis.
    pub async fn set<T: Serialize>(&self, key: &str, val: &T) -> Result<(), RedisError> {
        let json = serde_json::to_string(val)?;
        let () = self.client.set(key, json, None, None, false).await?;
        Ok(())
    }

    /// Stores a raw string in Redis.
    pub async fn set_str(&self, key: &str, val: &str) -> Result<(), RedisError> {
        let () = self.client.set(key, val, None, None, false).await?;
        Ok(())
    }

    /// Serializes an item to JSON and stores it in Redis with an expiration time in seconds.
    pub async fn set_ex<T: Serialize>(
        &self,
        key: &str,
        val: &T,
        ttl_secs: u64,
    ) -> Result<(), RedisError> {
        let json = serde_json::to_string(val)?;
        let expiration = Expiration::EX(ttl_secs as i64);
        let () = self
            .client
            .set(key, json, Some(expiration), None, false)
            .await?;
        Ok(())
    }

    /// Stores a raw string in Redis with an expiration time in seconds.
    pub async fn set_ex_str(&self, key: &str, val: &str, ttl_secs: u64) -> Result<(), RedisError> {
        let expiration = Expiration::EX(ttl_secs as i64);
        let () = self
            .client
            .set(key, val, Some(expiration), None, false)
            .await?;
        Ok(())
    }

    /// Deletes a key from Redis, returning true if the key existed and was deleted.
    pub async fn del(&self, key: &str) -> Result<bool, RedisError> {
        let count: i64 = self.client.del(key).await?;
        Ok(count > 0)
    }

    /// Checks if a key exists in Redis.
    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let count: i64 = self.client.exists(key).await?;
        Ok(count > 0)
    }

    /// Increments the integer value stored at key by one.
    pub async fn incr(&self, key: &str) -> Result<i64, RedisError> {
        let val: i64 = self.client.incr(key).await?;
        Ok(val)
    }

    /// Sets a timeout in seconds on a key.
    pub async fn expire(&self, key: &str, seconds: i64) -> Result<bool, RedisError> {
        let res: bool = self.client.expire(key, seconds, None).await?;
        Ok(res)
    }

    /// Returns the remaining time to live of a key in seconds.
    pub async fn ttl(&self, key: &str) -> Result<i64, RedisError> {
        let ttl: i64 = self.client.ttl(key).await?;
        Ok(ttl)
    }

    /// Executes a Lua script atomically on Redis.
    pub async fn eval<R: FromValue>(
        &self,
        script: &str,
        keys: Vec<&str>,
        args: Vec<&str>,
    ) -> Result<R, RedisError> {
        let res: R = self.client.eval(script, keys, args).await?;
        Ok(res)
    }
}
