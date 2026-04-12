mod db;
mod models;
mod repo;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("nexus_core=info".parse().unwrap()))
        .init();

    tracing::info!("NEXUS Core startet...");

    let pool = db::init_pool("sqlite:nexus.db")
        .await
        .expect("Datenbank konnte nicht initialisiert werden");

    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7777")
        .await
        .expect("Port 7777 konnte nicht gebunden werden");

    tracing::info!("NEXUS Core läuft auf http://127.0.0.1:7777");

    axum::serve(listener, app)
        .await
        .expect("Server-Fehler");
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok"}))
}
