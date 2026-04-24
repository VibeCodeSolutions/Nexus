mod auth;
mod cli;
mod config;
mod db;
mod handlers;
mod keystore;
mod llm;
mod models;
mod oauth;
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
        Command::Onboard => {
            if let Err(e) = run_onboard().await {
                eprintln!("Onboarding-Fehler: {e}");
            }
        }
        Command::Login { provider } => {
            if let Err(e) = run_oauth_login(&provider).await {
                eprintln!("Login-Fehler: {e}");
            }
        }
        Command::Logout { provider } => {
            let mut removed = false;
            if keystore::delete_oauth(&provider).is_ok() {
                println!("OAuth-Token für '{provider}' gelöscht.");
                removed = true;
            }
            if keystore::delete_key(&provider).is_ok() {
                println!("API-Key für '{provider}' gelöscht.");
                removed = true;
            }
            if !removed {
                println!("Nichts gefunden für '{provider}'.");
            }
        }
        Command::Status => {
            print_status();
        }
        Command::TestLlm { provider, text } => {
            let cfg = Config::load();
            let p = provider.unwrap_or(cfg.default_provider);
            if let Err(e) = run_test_llm(&p, &text).await {
                eprintln!("❌ Test fehlgeschlagen: {e}");
                std::process::exit(1);
            }
        }
        Command::Pair => {
            let config = Config::load();
            match auth::pairing_uri(&config.bind_addr) {
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
                    tracing::warn!("Server startet ohne LLM. Erstanmeldung: `nexus onboard` (Wizard) oder `nexus login claude` (OAuth).");
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
                .route("/braindump/{id}", delete(handlers::delete_braindump))
                .route("/braindump/recategorize", post(handlers::recategorize_unsorted))
                .route("/projects/suggest", post(handlers::suggest_projects))
                .route("/projects", post(handlers::create_project))
                .route("/projects", get(handlers::list_projects))
                .route("/projects/{id}", delete(handlers::delete_project))
                .route("/projects/{id}/braindumps", get(handlers::get_project_braindumps))
                .route("/projects/{id}/progress", get(handlers::get_project_progress))
                .route("/tasks", post(handlers::create_task))
                .route("/tasks", get(handlers::list_tasks))
                .route("/tasks/{id}", put(handlers::update_task))
                .route("/tasks/{id}", delete(handlers::delete_task))
                .route("/stats", get(handlers::get_stats))
                .route("/achievements", get(handlers::get_achievements))
                .route("/xp/history", get(handlers::get_xp_history))
                .route("/api/setup-status", get(handlers::setup_status))
                .route("/api/onboard/set-provider", post(handlers::onboard_set_provider))
                .route("/api/onboard/oauth", post(handlers::onboard_oauth))
                .route("/api/pair/uri", get(handlers::pair_uri))
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

async fn run_onboard() -> Result<(), String> {
    use dialoguer::{theme::ColorfulTheme, Input, Select};

    println!("\n🚀 NEXUS Onboarding\n");

    let providers = vec![
        "Claude (Anthropic)",
        "Gemini (Google)",
        "z.ai (GLM)",
        "OpenAI (ChatGPT)",
        "Mistral",
        "Groq",
        "DeepSeek",
        "OpenRouter",
    ];
    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Welchen LLM-Provider möchtest du nutzen?")
        .items(&providers)
        .default(2)
        .interact()
        .map_err(|e| e.to_string())?;

    let provider = match provider_idx {
        0 => "claude",
        1 => "gemini",
        2 => "zai",
        3 => "openai",
        4 => "mistral",
        5 => "groq",
        6 => "deepseek",
        7 => "openrouter",
        _ => "zai",
    };

    if provider == "claude" {
        let methods = vec![
            "OAuth (Claude Pro/Max Subscription) — empfohlen",
            "API-Key (Anthropic Console)",
        ];
        let method = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Anmeldeverfahren")
            .items(&methods)
            .default(0)
            .interact()
            .map_err(|e| e.to_string())?;

        if method == 0 {
            run_oauth_login("claude").await?;
        } else {
            let key: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Anthropic API-Key (sk-ant-...)")
                .interact_text()
                .map_err(|e| e.to_string())?;
            keystore::set_key("claude", key.trim())?;
            println!("✅ Claude API-Key gespeichert.");
        }
    } else if provider == "gemini" {
        println!("ℹ️  Gemini unterstützt nur API-Key (kein Consumer-OAuth).");
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Google AI Studio API-Key")
            .interact_text()
            .map_err(|e| e.to_string())?;
        keystore::set_key("gemini", key.trim())?;
        println!("✅ Gemini API-Key gespeichert.");
    } else if provider == "zai" {
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("z.ai API-Key")
            .interact_text()
            .map_err(|e| e.to_string())?;
        keystore::set_key("zai", key.trim())?;
        println!("✅ z.ai API-Key gespeichert.");
    } else {
        let (label, prompt) = match provider {
            "openai" => ("OpenAI", "OpenAI API-Key (sk-...)"),
            "mistral" => ("Mistral", "Mistral API-Key"),
            "groq" => ("Groq", "Groq API-Key (gsk_...)"),
            "deepseek" => ("DeepSeek", "DeepSeek API-Key (sk-...)"),
            "openrouter" => ("OpenRouter", "OpenRouter API-Key (sk-or-...)"),
            _ => (provider, "API-Key"),
        };
        println!("ℹ️  {label} nutzt den OpenAI-kompatiblen API-Key-Flow.");
        let key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact_text()
            .map_err(|e| e.to_string())?;
        keystore::set_key(provider, key.trim())?;
        println!("✅ {label} API-Key gespeichert.");
    }

    println!("\nFertig! Starte den Server mit: nexus serve");
    Ok(())
}

fn print_status() {
    let cfg = Config::load();
    println!("\n📊 NEXUS Status\n");
    println!("Default-Provider: {}", cfg.default_provider);
    println!("Bind:             {}", cfg.bind_addr);
    println!("DB:               {}", cfg.db_url);
    println!("Logs:             {}\n", cfg.log_dir.display());

    println!("Auth-Status:");
    for provider in &[
        "claude",
        "gemini",
        "zai",
        "ollama",
        "openai",
        "mistral",
        "groq",
        "deepseek",
        "openrouter",
    ] {
        let oauth = keystore::get_oauth(provider).ok();
        let api_key = keystore::get_key(provider).ok();

        let active = if oauth.is_some() {
            "OAuth ✅"
        } else if api_key.is_some() {
            "API-Key ✅"
        } else {
            "— nicht konfiguriert"
        };
        println!("  {provider:8} → aktiv: {active}");

        if let Some(t) = oauth {
            let now = chrono::Utc::now().timestamp();
            let secs = t.expires_at - now;
            let state = if secs <= 0 {
                "abgelaufen (wird beim nächsten Call refresht)".to_string()
            } else if secs < 60 {
                format!("läuft in {secs}s ab")
            } else {
                format!("gültig für {}min", secs / 60)
            };
            println!("           OAuth-Token: {state}");
        }
        if api_key.is_some() {
            println!("           API-Key: gespeichert");
        }
    }
    println!("\nTest: `nexus test-llm`  •  Wizard: `nexus onboard`\n");
}

async fn run_test_llm(provider: &str, text: &str) -> Result<(), String> {
    println!("🧪 Test-Call gegen Provider: {provider}");
    println!("   Input: {text:?}\n");

    let p = llm::create_provider(provider)?;
    let start = std::time::Instant::now();
    let result = p.categorize_and_summarize(text).await?;
    let ms = start.elapsed().as_millis();

    println!("✅ Antwort in {ms}ms:");
    println!("   Kategorie: {}", result.category);
    println!("   Summary:   {}", result.summary);
    println!("   Tags:      {:?}", result.tags);
    Ok(())
}

async fn run_oauth_login(provider: &str) -> Result<(), String> {
    use dialoguer::{theme::ColorfulTheme, Input};

    if provider != "claude" {
        return Err(format!("OAuth nicht verfügbar für '{provider}' — nur Claude."));
    }

    let pkce = oauth::generate_pkce();
    let state = oauth::random_state();
    let url = oauth::build_authorize_url(&pkce, &state);

    println!("\n🔐 Öffne Browser für Anthropic-Login...");
    if webbrowser::open(&url).is_err() {
        println!("Konnte Browser nicht öffnen. Öffne diese URL manuell:\n{url}\n");
    } else {
        println!("Falls der Browser nicht öffnet, nutze:\n{url}\n");
    }

    let code: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Code aus dem Browser hier einfügen")
        .interact_text()
        .map_err(|e| e.to_string())?;

    let tokens = oauth::exchange_code(code.trim(), &pkce.verifier, &state).await?;
    keystore::set_oauth("claude", &tokens)?;
    println!("✅ Claude OAuth erfolgreich. Token gespeichert.");
    Ok(())
}
