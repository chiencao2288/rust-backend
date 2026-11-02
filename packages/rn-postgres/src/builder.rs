use crate::client::RNPostgres;
use crate::error::PostgresError;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

/// Builder for configuring and creating an RNPostgres client with 1 Master and N Slaves.
#[derive(Debug, Clone)]
pub struct RNPostgresBuilder {
    master_url: Option<String>,
    slave_urls: Vec<String>,
    max_connections: u32,
    min_connections: u32,
    acquire_timeout: Duration,
    idle_timeout: Duration,
}

impl Default for RNPostgresBuilder {
    fn default() -> Self {
        Self {
            master_url: None,
            slave_urls: Vec::new(),
            max_connections: 20,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl RNPostgresBuilder {
    /// Creates a default builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Master database URL (can only be configured once).
    pub fn with_master_url(mut self, url: impl Into<String>) -> Self {
        self.master_url = Some(url.into());
        self
    }

    /// Adds a Slave replica database URL if present.
    pub fn add_slave_url(mut self, url: Option<impl Into<String>>) -> Self {
        if let Some(url) = url {
            self.slave_urls.push(url.into());
        }
        self
    }

    /// Sets multiple Slave replica database URLs.
    pub fn with_slave_urls(mut self, urls: &[impl AsRef<str>]) -> Self {
        self.slave_urls = urls.iter().map(|u| u.as_ref().to_string()).collect();
        self
    }

    /// Sets the maximum number of pool connections per pool.
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Sets the minimum number of pool connections per pool.
    pub fn with_min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Sets the connection acquisition timeout.
    pub fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// Sets the connection idle timeout.
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Builds and connects the Master and Slave connection pools.
    pub async fn build(self) -> Result<RNPostgres, PostgresError> {
        let master_url = self.master_url.ok_or_else(|| {
            PostgresError::Configuration("Master database URL is required".into())
        })?;

        let pool_options = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(self.acquire_timeout)
            .idle_timeout(self.idle_timeout);

        let master_pool = pool_options.clone().connect(&master_url).await?;

        let mut slave_pools = Vec::with_capacity(self.slave_urls.len());
        for slave_url in &self.slave_urls {
            if slave_url != &master_url {
                let pool = pool_options.clone().connect(slave_url).await?;
                slave_pools.push(pool);
            }
        }

        Ok(RNPostgres::new(master_pool, slave_pools))
    }
}
