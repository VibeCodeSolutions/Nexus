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
    /// Sprint Nightvision (NV-1): Foto-Braindump-Konfiguration.
    /// Konsument folgt in NV-2 (HTTP-Endpoint) — bis dahin dead_code.
    #[allow(dead_code)]
    pub vision: VisionConfig,
    /// Verzeichnis für persistierte Foto-Braindump-Bilder. Default:
    /// `<home>/.nexus/braindump_images/`. Override via NEXUS_BRAINDUMP_IMAGES_DIR.
    /// Konsument folgt in NV-2 — bis dahin dead_code.
    #[allow(dead_code)]
    pub braindump_images_dir: PathBuf,
}

/// Konfiguration für Vision-LLM + OCR-Fallback (Sprint Nightvision NV-1).
///
/// Provider-Key wird wie bei `LlmProvider` aus dem Keystore geholt — z. B.
/// `nexus-core set-key groq <key>` für `provider = "groq"`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VisionConfig {
    /// Aktivierungs-Flag. Wenn `false`, lehnt der Foto-Endpoint die
    /// Anfrage ab (= Kamera-Analyse-Toggle aus der Settings-Spec).
    /// Override via NEXUS_VISION_ENABLED.
    pub enabled: bool,
    /// Provider-Name (analog `LlmProvider::create_provider`). Vision-fähig
    /// sind aktuell: `groq` (llama-3.2-vision), `claude`, `gemini`, `openai`,
    /// `xai` (grok-vision), `openrouter`. Override via NEXUS_VISION_PROVIDER.
    pub provider: String,
    /// Konkretes Vision-Modell — überschreibt das Default-Modell des Providers.
    /// Override via NEXUS_VISION_MODEL.
    pub model: Option<String>,
    /// Tesseract-Fallback: wenn der konfigurierte Vision-Provider fehlschlägt
    /// oder kein Key gesetzt ist, wird `tesseract` als Subprocess aufgerufen
    /// und das Ergebnis durch den klassischen LlmProvider für Tag-Generation
    /// gejagt. Override via NEXUS_OCR_TESSERACT_ENABLED.
    pub tesseract_enabled: bool,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "groq".to_string(),
            model: None,
            tesseract_enabled: true,
        }
    }
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

        // Sprint Nightvision NV-1: Vision-Config aus Environment ableiten.
        let vision = VisionConfig {
            enabled: env_bool("NEXUS_VISION_ENABLED", false),
            provider: env::var("NEXUS_VISION_PROVIDER")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "groq".to_string()),
            model: env::var("NEXUS_VISION_MODEL")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            tesseract_enabled: env_bool("NEXUS_OCR_TESSERACT_ENABLED", true),
        };

        let braindump_images_dir = env::var("NEXUS_BRAINDUMP_IMAGES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".nexus")
                    .join("braindump_images")
            });

        Self {
            default_provider,
            db_path,
            bind_addr: env::var("NEXUS_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:7777".to_string()),
            log_dir,
            vault_path,
            vision,
            braindump_images_dir,
        }
    }
}

/// Parsed `1`/`true`/`yes`/`on` (case-insensitive) als true, `0`/`false`/`no`/`off`
/// als false. Leerstring / unbekannte Werte → `default`.
fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key).ok().as_deref().map(|s| s.trim().to_ascii_lowercase()) {
        Some(ref v) if v == "1" || v == "true" || v == "yes" || v == "on" => true,
        Some(ref v) if v == "0" || v == "false" || v == "no" || v == "off" => false,
        _ => default,
    }
}

pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}
