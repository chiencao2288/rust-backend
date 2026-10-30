use crate::builder::RNPostgresBuilder;
use crate::error::PostgresError;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// PostgreSQL client managing 1 Master and multiple Slave replica connection pools.
#[derive(Clone, Debug)]
pub struct RNPostgres {
    master: PgPool,
    slaves: Arc<Vec<PgPool>>,
    counter: Arc<AtomicUsize>,
}

impl RNPostgres {
    /// Creates a new RNPostgres instance from existing master and slave pools.
    pub fn new(master: PgPool, slaves: Vec<PgPool>) -> Self {
        Self {
            master,
            slaves: Arc::new(slaves),
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Creates a new builder instance.
    pub fn builder() -> RNPostgresBuilder {
        RNPostgresBuilder::new()
    }

    /// Connects to a single PostgreSQL database URL as Master.
    pub async fn connect(url: &str) -> Result<Self, PostgresError> {
        Self::builder().with_master_url(url).build().await
    }

    /// Returns a reference to the primary (Master) connection pool for write operations.
    pub fn master(&self) -> &PgPool {
        &self.master
    }

    /// Returns a reference to a Slave pool via round-robin, falling back to Master if no slaves exist.
    pub fn slave(&self) -> &PgPool {
        if self.slaves.is_empty() {
            return &self.master;
        }
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % self.slaves.len();
        &self.slaves[index]
    }

    /// Begins a new transaction on the Master connection pool.
    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>, PostgresError> {
        let tx = self.master.begin().await?;
        Ok(tx)
    }

    /// Pings master and all slave pools to verify connectivity.
    pub async fn ping(&self) -> Result<(), PostgresError> {
        sqlx::query("SELECT 1").execute(&self.master).await?;
        for slave in self.slaves.iter() {
            sqlx::query("SELECT 1").execute(slave).await?;
        }
        Ok(())
    }

    /// Checks if a database error is a unique constraint violation (PostgreSQL code 23505).
    pub fn is_unique_violation(err: &sqlx::Error) -> bool {
        if let sqlx::Error::Database(db_err) = err {
            return db_err.code().as_deref() == Some("23505");
        }
        false
    }

    /// Checks if a database error is a foreign key constraint violation (PostgreSQL code 23503).
    pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
        if let sqlx::Error::Database(db_err) = err {
            return db_err.code().as_deref() == Some("23503");
        }
        false
    }
}
