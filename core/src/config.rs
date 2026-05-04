use std::env;
use std::path::PathBuf;

pub struct Config {
    pub default_provider: String,
    pub db_path: PathBuf,
    pub bind_addr: String,
    pub log_dir: PathBuf,
    /// Optionaler Obsidian-Vault-Pfad. Wenn gesetzt, schreibt Nexus
    /// BrainDumps in `<vault>/Nexus/Inbox/` und liest sortierte Files
    /// aus `<vault>/Nexus/Outbox/`. Präzedenz: NEXUS_VAULT_PATH > Keystore.
    pub vault_path: Option<PathBuf>,
}

// Phase B/C konsumieren diese Konstanten/Helper. Bis dahin würden sie
// als dead-code warnen — `#[allow]` analog zu `clear_default_provider`
// in keystore.rs.
#[allow(dead_code)]
impl Config {
    /// Standard-Subpfade unterhalb des Vault-Roots.
    pub const INBOX_SUBDIR: &'static str = "Nexus/Inbox";
    pub const OUTBOX_SUBDIR: &'static str = "Nexus/Outbox";
    pub const OUTBOX_PROCESSED_SUBDIR: &'static str = "Nexus/Outbox/_processed";

    pub fn inbox_dir(&self) -> Option<PathBuf> {
        self.vault_path.as_ref().map(|v| v.join(Self::INBOX_SUBDIR))
    }

    pub fn outbox_dir(&self) -> Option<PathBuf> {
        self.vault_path.as_ref().map(|v| v.join(Self::OUTBOX_SUBDIR))
    }

    pub fn outbox_processed_dir(&self) -> Option<PathBuf> {
        self.vault_path
            .as_ref()
            .map(|v| v.join(Self::OUTBOX_PROCESSED_SUBDIR))
    }
}

impl Config {
    pub fn load() -> Self {
        let log_dir = env::var("NEXUS_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus").join("logs")
            });

        // Präzedenz: keystore > env > hardcoded "gemini"
        let env_provider = env::var("NEXUS_DEFAULT_PROVIDER")
            .unwrap_or_else(|_| "gemini".to_string());
        let default_provider = crate::keystore::get_default_provider()
            .unwrap_or(env_provider);

        // DB-Pfad: absolut in `~/.nexus/nexus.db`. Vorher relativer
        // Pfad `nexus.db` → CWD-abhängig, daher landeten Daten in
        // verschiedenen Files je nach Start-Modus (Tauri-Sidecar CWD
        // ≠ Standalone-CLI CWD). Override via NEXUS_DB_URL nimmt einen
        // Filesystem-Pfad (mit oder ohne `sqlite:` / `sqlite://` Prefix).
        // In-Memory-DB ist nur über `db::init_in_memory` im Test-Code
        // verfügbar — `NEXUS_DB_URL=sqlite::memory:` würde sich seit
        // diesem Refactor als Pfad `:memory:` interpretieren.
        let db_path = env::var("NEXUS_DB_URL")
            .map(|s| {
                // Strip `sqlite:` / `sqlite://` prefix wenn vorhanden,
                // damit alte Configs mit URL-Form weiter funktionieren.
                let trimmed = s.strip_prefix("sqlite://").or_else(|| s.strip_prefix("sqlite:")).unwrap_or(&s);
                PathBuf::from(trimmed)
            })
            .unwrap_or_else(|_| {
                home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus").join("nexus.db")
            });

        // Vault-Pfad: env > keystore > None. Leerstring im env wird als
        // „nicht gesetzt" behandelt, damit `NEXUS_VAULT_PATH=""` ein
        // Keystore-Setting nicht unbeabsichtigt überschreibt.
        let vault_path = env::var("NEXUS_VAULT_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .or_else(|| crate::keystore::get_vault_path().map(PathBuf::from));

        Self {
            default_provider,
            db_path,
            bind_addr: env::var("NEXUS_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:7777".to_string()),
            log_dir,
            vault_path,
        }
    }
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}
