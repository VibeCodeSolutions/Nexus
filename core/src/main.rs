mod auth;
mod cli;
mod config;
mod db;
mod handlers;
mod keystore;
mod llm;
mod models;
mod repo;

use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use clap::Parser;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::Config;
use llm::LlmProvider;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub llm: Arc<dyn LlmProvider>,
}

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
        Command::Pair => {
            let config = Config::load();
            match auth::pairing_json(&config.bind_addr) {
                Ok(data) => auth::print_qr(&data),
                Err(e) => eprintln!("Fehler: {e}"),
            }
        }
        Command::Serve => {
            let config = Config::load();

            // Log-Verzeichnis erstellen
            std::fs::create_dir_all(&config.log_dir).ok();

            // File-Logger mit täglicher Rotation
            let file_appender = tracing_appender::rolling::daily(&config.log_dir, "nexus.log");
            let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

            let filter = EnvFilter::from_default_env()
                .add_directive("nexus_core=info".parse().unwrap());

            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
                .with(tracing_subscriber::fmt::layer().with_writer(file_writer).with_ansi(false))
                .init();

            tracing::info!("NEXUS Core startet... (Provider: {})", config.default_provider);
            tracing::info!("Logs: {}", config.log_dir.display());

            // Ensure pairing token exists
            match auth::get_or_create_token() {
                Ok(_) => tracing::info!("Pairing-Token bereit. QR-Code anzeigen mit: nexus pair"),
                Err(e) => tracing::warn!("Token konnte nicht erstellt werden: {e}"),
            }

            let pool = db::init_pool(&config.db_url)
                .await
                .expect("Datenbank konnte nicht initialisiert werden");

            let llm_provider: Arc<dyn LlmProvider> = match llm::create_provider(&config.default_provider) {
                Ok(provider) => Arc::from(provider),
                Err(e) => {
                    tracing::warn!("LLM-Provider nicht verfügbar: {e}");
                    tracing::warn!("Server startet ohne LLM. Keys setzen mit: nexus set-key claude <key>");
                    Arc::new(llm::NoOpProvider)
                }
            };

            let state = AppState {
                pool,
                llm: llm_provider,
            };

            let app = Router::new()
                .route("/", get(handlers::dashboard))
                .route("/health", get(health_check))
                .route("/braindump", post(handlers::post_braindump))
                .route("/braindump", get(handlers::list_braindumps))
                .route("/braindump/{id}", get(handlers::get_braindump))
                .route("/projects/suggest", post(handlers::suggest_projects))
                .route("/projects", post(handlers::create_project))
                .route("/projects", get(handlers::list_projects))
                .route("/projects/{id}/braindumps", get(handlers::get_project_braindumps))
                .route("/projects/{id}/progress", get(handlers::get_project_progress))
                .route("/tasks", post(handlers::create_task))
                .route("/tasks", get(handlers::list_tasks))
                .route("/tasks/{id}", put(handlers::update_task))
                .route("/tasks/{id}", delete(handlers::delete_task))
                .layer(middleware::from_fn(auth::require_token))
                .with_state(state);

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
