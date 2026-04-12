use std::env;
use std::path::PathBuf;

pub struct Config {
    pub default_provider: String,
    pub db_url: String,
    pub bind_addr: String,
    pub log_dir: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let log_dir = env::var("NEXUS_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus").join("logs")
            });

        Self {
            default_provider: env::var("NEXUS_DEFAULT_PROVIDER")
                .unwrap_or_else(|_| "claude".to_string()),
            db_url: env::var("NEXUS_DB_URL")
                .unwrap_or_else(|_| "sqlite:nexus.db".to_string()),
            bind_addr: env::var("NEXUS_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:7777".to_string()),
            log_dir,
        }
    }
}

pub fn home_dir() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}
