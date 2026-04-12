mod cli;
mod config;
mod db;
mod keystore;
mod llm;
mod models;
mod repo;

use axum::{routing::get, Json, Router};
use clap::Parser;
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::Config;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Serve) {
        Command::SetKey { provider, value } => {
            match keystore::set_key(&provider, &value) {
                Ok(()) => println!("API-Key für '{provider}' gespeichert."),
                Err(e) => eprintln!("Fehler: {e}"),
            }
        }
        Command::Serve => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::from_default_env()
                        .add_directive("nexus_core=info".parse().unwrap()),
                )
                .init();

            let config = Config::load();
            tracing::info!("NEXUS Core startet... (Provider: {})", config.default_provider);

            let pool = db::init_pool(&config.db_url)
                .await
                .expect("Datenbank konnte nicht initialisiert werden");

            let app = Router::new()
                .route("/health", get(health_check))
                .with_state(pool);

            let listener = tokio::net::TcpListener::bind(&config.bind_addr)
                .await
                .expect("Port konnte nicht gebunden werden");

            tracing::info!("NEXUS Core läuft auf http://{}", config.bind_addr);

            axum::serve(listener, app)
                .await
                .expect("Server-Fehler");
        }
    }
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok"}))
}
