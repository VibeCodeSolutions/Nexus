mod auth;
mod cli;
mod config;
mod db;
mod diag;
mod handlers;
mod keystore;
mod links;
mod llm;
mod models;
mod oauth;
mod obsidian;
mod repo;
mod suggestions;

use axum::middleware;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use tower_http::cors::{Any, CorsLayer};
use clap::Parser;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::net::SocketAddr;
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
    pub started_at: std::time::Instant,
    /// Singleflight-Lock für `/api/obsidian/sync` (OB-C-MIN-5).
    /// `try_lock` im Handler → bei laufendem Sync 409 CONFLICT, statt
    /// parallel zwei Importer auf denselben Outbox-Files anzusetzen
    /// (würde Doppel-Inserts erzeugen, weil aktuell kein nexus_id-Dedup
    /// auf Task/Project-Inserts existiert — siehe OB-C-MIN-4 für Phase E).
    pub obsidian_sync_lock: Arc<tokio::sync::Mutex<()>>,
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
                .add_directive("nexus_core=info".parse().unwrap())
                .add_directive("nexus_core::auth=debug".parse().unwrap());

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

            let pool = db::init_pool(&config.db_path)
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
                started_at: std::time::Instant::now(),
                obsidian_sync_lock: Arc::new(tokio::sync::Mutex::new(())),
            };

            // Clone für Background-Recategorize-Task (Phase D / JJ-D2) — VOR with_state(state)
            let bg_pool = state.pool.clone();
            let bg_llm = state.llm.clone();

            let app = Router::new()
                .route("/", get(handlers::dashboard))
                .route("/health", get(health_check))
                .route("/braindump", post(handlers::post_braindump))
                .route("/braindump", get(handlers::list_braindumps))
                .route("/braindump/{id}", get(handlers::get_braindump))
                .route("/braindump/{id}", delete(handlers::delete_braindump))
                .route("/braindump/recategorize", post(handlers::recategorize_unsorted))
                .route("/braindump/unsorted/count", get(handlers::unsorted_count))
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
                .route("/api/settings/providers", get(handlers::settings_providers))
                .route("/api/settings/models", get(handlers::settings_models))
                .route("/api/settings/provider", post(handlers::settings_set_provider))
                .route("/api/pair/uri", get(handlers::pair_uri))
                .route("/api/pair/handshake", post(handlers::pair_handshake))
                .route("/api/diag/run", post(handlers::diag_run))
                .route("/api/diag/report", post(handlers::diag_report))
                .route("/api/diag/reports", get(handlers::diag_list))
                // Synaptic Mosaic Phase B
                .route("/links", post(handlers::create_link))
                .route("/links/{id}", delete(handlers::delete_link))
                .route("/braindump/{id}/links", get(handlers::get_braindump_links))
                .route("/projects/{id}/links", get(handlers::get_project_links))
                .route("/projects/suggestions", get(handlers::list_project_suggestions))
                .route("/projects/suggestions/{id}/accept", post(handlers::accept_project_suggestion))
                .route("/projects/suggestions/{id}/dismiss", post(handlers::dismiss_project_suggestion))
                // Obsidian-Briefkasten Phase C
                .route("/api/obsidian/sync", post(handlers::obsidian_sync))
                .layer(middleware::from_fn(auth::require_token))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                )
                .with_state(state);

            // Single-Core-Garant: prüfen ob bereits ein Prozess auf diesem Port lauscht.
            // Verhindert Doppelstart von Sidecar + systemd-Service.
            let port = config.bind_addr.split(':').next_back().unwrap_or("7777");
            let probe = tokio::time::timeout(
                std::time::Duration::from_millis(200),
                tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")),
            )
            .await;
            if matches!(probe, Ok(Ok(_))) {
                tracing::error!(
                    "Port {port} ist bereits belegt — ein anderer NEXUS-Core lauscht. Abbruch (kein Doppelstart)."
                );
                eprintln!(
                    "Fehler: Ein NEXUS-Core läuft bereits auf Port {port}. Bitte den anderen Prozess stoppen, bevor du einen neuen startest."
                );
                return;
            }

            let listener = tokio::net::TcpListener::bind(&config.bind_addr)
                .await
                .expect("Port konnte nicht gebunden werden");

            tracing::info!("NEXUS Core läuft auf http://{}", config.bind_addr);
            match local_ip_address::local_ip() {
                Ok(ip) => {
                    let port = config.bind_addr.split(':').next_back().unwrap_or("7777");
                    tracing::info!(
                        "Mobile Pairing erwartet IP http://{}:{} — Pixel muss im selben LAN sein",
                        ip,
                        port
                    );
                }
                Err(e) => tracing::warn!(
                    "LAN-IP konnte nicht ermittelt werden ({}); Pairing-QR fällt auf 127.0.0.1 zurück und ist mobil unbrauchbar",
                    e
                ),
            }

            // Background-Recategorize-Task (Phase D / JJ-D2 + SM Phase B)
            // bg_pool und bg_llm wurden bereits vor with_state(state) geklont.
            let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
            tokio::spawn(async move {
                let base_delay: u64 = std::env::var("NEXUS_RECATEGORIZE_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300);
                let max_delay: u64 = 3600;
                let mut delay_secs = base_delay;
                // SM-PR-004: Auto-Projekt-Trigger alle N Recategorize-Cycles (default 6 ≈ 30min)
                let auto_project_every: u64 = std::env::var("NEXUS_AUTO_PROJECT_INTERVAL_CYCLES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(6);
                let mut cycle: u64 = 0;
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(delay_secs)) => {}
                        _ = cancel_rx.changed() => {
                            tracing::info!("recategorize-bg: shutdown");
                            break;
                        }
                    }
                    cycle = cycle.wrapping_add(1);
                    let mut had_failure = false;
                    match handlers::recategorize_unsorted_inner(&bg_pool, bg_llm.as_ref(), 50).await {
                        Ok(stats) if stats.failed == 0 => {
                            if stats.updated > 0 || stats.total > 0 {
                                tracing::info!(
                                    "recategorize-bg: {}/{} updated",
                                    stats.updated,
                                    stats.total
                                );
                            }
                        }
                        Ok(stats) => {
                            tracing::warn!(
                                "recategorize-bg: {} failed of {} — backoff",
                                stats.failed,
                                stats.total
                            );
                            had_failure = true;
                        }
                        Err(e) => {
                            tracing::warn!("recategorize-bg DB error: {e} — backoff");
                            had_failure = true;
                        }
                    }

                    // SM Phase B-6a: extract_links_for_recent jeden Cycle
                    match handlers::extract_links_for_recent(&bg_pool, bg_llm.as_ref(), 10).await {
                        Ok(stats) => {
                            if stats.links_created > 0 || stats.failed > 0 {
                                tracing::info!(
                                    "extract-links-bg: {} links_created, {} processed, {} failed",
                                    stats.links_created, stats.processed, stats.failed
                                );
                            }
                            if stats.failed > 0 { had_failure = true; }
                        }
                        Err(e) => {
                            tracing::warn!("extract-links-bg DB error: {e}");
                            had_failure = true;
                        }
                    }

                    // SM Phase B-6b: suggest_auto_projects nur jeden N-ten Cycle
                    if cycle.is_multiple_of(auto_project_every) {
                        match handlers::suggest_auto_projects(&bg_pool, bg_llm.as_ref()).await {
                            Ok(stats) => {
                                if stats.auto_created > 0 || stats.suggestions_added > 0 {
                                    tracing::info!(
                                        "auto-project-bg: {} auto_created, {} suggestions, {} considered",
                                        stats.auto_created, stats.suggestions_added, stats.considered
                                    );
                                }
                                if stats.failed > 0 { had_failure = true; }
                            }
                            Err(e) => {
                                tracing::warn!("auto-project-bg DB error: {e}");
                                had_failure = true;
                            }
                        }
                    }

                    if had_failure {
                        delay_secs = (delay_secs.saturating_mul(3)).min(max_delay);
                    } else {
                        delay_secs = base_delay;
                    }
                }
            });

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .expect("Server-Fehler");
            // Cancel-Token explizit triggern, damit Background-Task sauber endet.
            let _ = cancel_tx.send(true);
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
        "Obsidian-Briefkasten (File-Bridge, kein Cloud-LLM)",
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
        8 => "obsidian",
        _ => "zai",
    };

    if provider == "obsidian" {
        println!(
            "ℹ️  Obsidian-Briefkasten: Nexus schreibt BrainDumps als Markdown-Dateien\n   in <Vault>/Nexus/Inbox/. Ein Vault-seitiges Sortier-Skill erzeugt\n   Outbox-Dateien, die Nexus per `POST /api/obsidian/sync` zurückzieht.\n"
        );
        let vault_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Absoluter Pfad zum Obsidian-Vault")
            .interact_text()
            .map_err(|e| e.to_string())?;
        let trimmed = vault_input.trim();
        if trimmed.is_empty() {
            return Err("Vault-Pfad darf nicht leer sein".to_string());
        }
        let path = std::path::Path::new(trimmed);
        if !path.is_dir() {
            return Err(format!(
                "Vault-Pfad '{trimmed}' existiert nicht oder ist kein Verzeichnis"
            ));
        }
        keystore::set_vault_path(trimmed)?;
        keystore::set_default_provider("obsidian")?;
        println!("✅ Obsidian-Vault gesetzt: {trimmed}");
        println!("\nFertig! Starte den Server mit: nexus serve");
        return Ok(());
    }

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
    println!("DB:               {}", cfg.db_path.display());
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
