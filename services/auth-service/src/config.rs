use std::env;

/// Application configuration with strict fail-fast validation.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_master_url: String,
    pub database_slave_urls: Vec<String>,
    pub redis_nodes: Vec<String>,
    pub redis_password: Option<String>,
    pub session_ttl_secs: u64,
    pub refresh_token_ttl_days: i64,
    pub google_client_id: Option<String>,
}

impl Config {
    /// Loads and validates configuration from environment, terminating the process on any missing required variable.
    pub fn load_or_exit() -> Self {
        match Self::load() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("\n=======================================================");
                eprintln!(" [FATAL CONFIG ERROR] Service startup aborted!");
                eprintln!(" Reason: {err}");
                eprintln!("=======================================================\n");
                std::process::exit(1);
            }
        }
    }

    /// Loads configuration, returning an error if any required variable is absent or invalid.
    pub fn load() -> Result<Self, String> {
        let port = match env::var("PORT") {
            Ok(val) => val
                .parse::<u16>()
                .map_err(|e| format!("Invalid PORT environment variable '{val}': {e}"))?,
            Err(_) => 8080,
        };

        let database_master_url = env::var("DATABASE_MASTER_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .map_err(|_| {
                "Missing required environment variable: DATABASE_MASTER_URL (or DATABASE_URL)"
                    .to_string()
            })?;

        if database_master_url.trim().is_empty() {
            return Err("DATABASE_MASTER_URL cannot be empty".into());
        }

        let database_slave_urls = env::var("DATABASE_SLAVE_URLS")
            .map(|s| {
                s.split(',')
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let redis_nodes_raw = env::var("REDIS_NODES")
            .or_else(|_| env::var("REDIS_URL"))
            .map_err(|_| {
                "Missing required environment variable: REDIS_NODES (or REDIS_URL)".to_string()
            })?;

        let redis_nodes: Vec<String> = redis_nodes_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if redis_nodes.is_empty() {
            return Err("REDIS_NODES must contain at least one valid node URL".into());
        }

        let redis_password = env::var("REDIS_PASSWORD").ok().filter(|s| !s.is_empty());

        let session_ttl_secs = match env::var("SESSION_TTL_SECS") {
            Ok(val) => val
                .parse::<u64>()
                .map_err(|e| format!("Invalid SESSION_TTL_SECS '{val}': {e}"))?,
            Err(_) => 1800,
        };

        let refresh_token_ttl_days = match env::var("REFRESH_TOKEN_TTL_DAYS") {
            Ok(val) => val
                .parse::<i64>()
                .map_err(|e| format!("Invalid REFRESH_TOKEN_TTL_DAYS '{val}': {e}"))?,
            Err(_) => 30,
        };

        let google_client_id = env::var("GOOGLE_CLIENT_ID").ok().filter(|s| !s.is_empty());

        Ok(Self {
            port,
            database_master_url,
            database_slave_urls,
            redis_nodes,
            redis_password,
            session_ttl_secs,
            refresh_token_ttl_days,
            google_client_id,
        })
    }
}
