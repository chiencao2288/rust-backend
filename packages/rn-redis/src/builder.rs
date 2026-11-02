use crate::client::RNRedis;
use crate::error::RedisError;
use fred::prelude::*;
use std::time::Duration;

/// Builder for configuring and creating an RNRedis client.
#[derive(Debug, Clone)]
pub struct RNRedisBuilder {
    nodes: Vec<String>,
    connection_timeout: Duration,
    password: Option<String>,
}

impl Default for RNRedisBuilder {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            connection_timeout: Duration::from_secs(5),
            password: None,
        }
    }
}

impl RNRedisBuilder {
    /// Creates a default builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a single Redis node URL if present.
    pub fn add_node(mut self, url: Option<impl Into<String>>) -> Self {
        if let Some(url) = url {
            self.nodes.push(url.into());
        }
        self
    }

    /// Sets the list of Redis node URLs.
    pub fn with_nodes(mut self, urls: &[impl AsRef<str>]) -> Self {
        self.nodes = urls.iter().map(|u| u.as_ref().to_string()).collect();
        self
    }

    /// Sets the connection timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Sets an optional cluster password override if present.
    pub fn with_password(mut self, password: Option<impl Into<String>>) -> Self {
        if let Some(pwd) = password {
            self.password = Some(pwd.into());
        }
        self
    }

    /// Builds and connects the Redis client (automatically supporting standalone and cluster deployments).
    pub async fn build(self) -> Result<RNRedis, RedisError> {
        if self.nodes.is_empty() {
            return Err(RedisError::Configuration(
                "At least one Redis node URL is required".into(),
            ));
        }

        let mut config = if self.nodes.len() > 1 {
            let mut cfg = Config::from_url_clustered(&self.nodes[0])?;
            if let ServerConfig::Clustered { ref mut hosts, .. } = cfg.server {
                for node in self.nodes.iter().skip(1) {
                    let other_cfg = Config::from_url_clustered(node)?;
                    if let ServerConfig::Clustered {
                        hosts: other_hosts, ..
                    } = other_cfg.server
                    {
                        hosts.extend(other_hosts);
                    }
                }
            }
            cfg
        } else {
            Config::from_url(&self.nodes[0])?
        };

        if let Some(pwd) = self.password {
            config.password = Some(pwd);
        }

        let client = Builder::from_config(config)
            .with_connection_config(|cfg| {
                cfg.connection_timeout = self.connection_timeout;
            })
            .build()?;

        client.init().await?;

        Ok(RNRedis::from_client(client))
    }
}
