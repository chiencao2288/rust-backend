use rn_bootstrap::EnvConfig;

/// Application configuration with strict fail-fast validation.
#[derive(Debug, Clone, EnvConfig)]
pub struct Config {
    #[env("PORT", fallback = "8081")]
    pub port: u16,

    #[env("DATABASE_MASTER_URL")]
    pub database_master_url: String,

    #[env("DATABASE_SLAVE_URL_01")]
    pub database_slave_url_01: Option<String>,

    #[env("REDIS_NODE_01")]
    pub redis_node_01: String,

    #[env("REDIS_PASSWORD")]
    pub redis_password: Option<String>,

    #[env("RABBITMQ_URL")]
    pub rabbitmq_url: String,

    #[env("AUTH_SERVICE_URL", fallback = "http://127.0.0.1:8080")]
    pub auth_service_url: String,

    #[env("EMAIL_PROVIDER", fallback = "smtp")]
    pub email_provider: String,

    #[env("MAIL_HOST")]
    pub mail_host: Option<String>,

    #[env("MAIL_PORT", fallback = "587")]
    pub mail_port: u16,

    #[env("MAIL_SECURE", fallback = "false")]
    pub mail_secure: String,

    #[env("MAIL_USER")]
    pub mail_user: Option<String>,

    #[env("MAIL_PASS")]
    pub mail_pass: Option<String>,

    #[env("MAIL_FROM", fallback = "chiencao19920808@gmail.com")]
    pub mail_from: String,

    #[env("MAIL_FROM_NAME", fallback = "Podkeen")]
    pub mail_from_name: String,
    #[env("SMS_PROVIDER", fallback = "mock")]
    pub sms_provider: String,
}
