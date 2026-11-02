use rn_bootstrap::EnvConfig;

/// Application configuration with strict fail-fast validation.
#[derive(Debug, Clone, EnvConfig)]
pub struct Config {
    #[env("PORT", fallback = "8080")]
    pub port: u16,

    #[env("DATABASE_MASTER_URL")]
    pub database_master_url: String,

    #[env("DATABASE_SLAVE_URL_01")]
    pub database_slave_url_01: Option<String>,

    #[env("REDIS_NODE_01")]
    pub redis_node_01: String,

    #[env("REDIS_PASSWORD")]
    pub redis_password: Option<String>,

    #[env("SESSION_TTL_SEC", fallback = "1800")]
    pub session_ttl_sec: u64,

    #[env("REFRESH_TOKEN_TTL_DAY", fallback = "30")]
    pub refresh_token_ttl_day: i64,

    #[env("GOOGLE_CLIENT_ID")]
    pub google_client_id: Option<String>,

    #[env(
        "GOOGLE_OAUTH_URL",
        fallback = "https://oauth2.googleapis.com/tokeninfo"
    )]
    pub google_oauth_url: String,
}
