use std::env;

pub struct Config {
    pub default_provider: String,
    pub db_url: String,
    pub bind_addr: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            default_provider: env::var("NEXUS_DEFAULT_PROVIDER")
                .unwrap_or_else(|_| "claude".to_string()),
            db_url: env::var("NEXUS_DB_URL")
                .unwrap_or_else(|_| "sqlite:nexus.db".to_string()),
            bind_addr: env::var("NEXUS_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:7777".to_string()),
        }
    }
}
