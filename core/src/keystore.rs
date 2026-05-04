use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const VALID_PROVIDERS: &[&str] = &[
    "claude",
    "gemini",
    "zai",
    "ollama",
    "openai",
    "mistral",
    "groq",
    "deepseek",
    "openrouter",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    keys: HashMap<String, String>,
    oauth: HashMap<String, OAuthTokens>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_provider: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    models: HashMap<String, String>,
    /// Optionaler Obsidian-Vault-Pfad (User-Wahl im Wizard). Wird
    /// mit dem Rest der Konfig in `~/.nexus/keys.json` (mode 0o600)
    /// persistiert, damit die Setting-Hoheit beim User bleibt und
    /// nicht in einer separaten Config-Datei dupliziert wird.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    vault_path: Option<String>,
}

fn store_path() -> PathBuf {
    crate::config::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".nexus")
        .join("keys.json")
}

fn load() -> Store {
    let path = store_path();
    if !path.exists() {
        return Store::default();
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

fn save(store: &Store) -> Result<(), String> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Verzeichnis konnte nicht erstellt werden: {e}"))?;
    }
    let data = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Serialisierung fehlgeschlagen: {e}"))?;
    fs::write(&path, &data).map_err(|e| format!("Speichern fehlgeschlagen: {e}"))?;
    #[cfg(unix)]
    {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Permissions konnten nicht gesetzt werden: {e}"))?;
    }
    Ok(())
}

pub fn set_key(provider: &str, value: &str) -> Result<(), String> {
    if !VALID_PROVIDERS.contains(&provider) {
        return Err(format!(
            "Unbekannter Provider: {provider}. Erlaubt: {}",
            VALID_PROVIDERS.join(", ")
        ));
    }
    let mut store = load();
    store.keys.insert(provider.to_string(), value.to_string());
    save(&store)
}

pub fn get_key(provider: &str) -> Result<String, String> {
    load()
        .keys
        .get(provider)
        .cloned()
        .ok_or_else(|| format!("Key für '{provider}' nicht gefunden: No matching entry found in secure storage"))
}

pub fn delete_key(provider: &str) -> Result<(), String> {
    let mut store = load();
    store.keys.remove(provider);
    save(&store)
}

pub fn set_oauth(provider: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let mut store = load();
    store.oauth.insert(provider.to_string(), tokens.clone());
    save(&store)
}

pub fn get_oauth(provider: &str) -> Result<OAuthTokens, String> {
    load()
        .oauth
        .get(provider)
        .cloned()
        .ok_or_else(|| format!("OAuth-Token für '{provider}' nicht gefunden"))
}

pub fn delete_oauth(provider: &str) -> Result<(), String> {
    let mut store = load();
    store.oauth.remove(provider);
    save(&store)
}

pub fn set_default_provider(provider: &str) -> Result<(), String> {
    if !VALID_PROVIDERS.contains(&provider) {
        return Err(format!(
            "Unbekannter Provider: {provider}. Erlaubt: {}",
            VALID_PROVIDERS.join(", ")
        ));
    }
    let mut store = load();
    store.default_provider = Some(provider.to_string());
    save(&store)
}

pub fn get_default_provider() -> Option<String> {
    load().default_provider
}

#[allow(dead_code)]
pub fn clear_default_provider() -> Result<(), String> {
    let mut store = load();
    store.default_provider = None;
    save(&store)
}

pub fn set_model(provider: &str, model: &str) -> Result<(), String> {
    if !VALID_PROVIDERS.contains(&provider) {
        return Err(format!(
            "Unbekannter Provider: {provider}. Erlaubt: {}",
            VALID_PROVIDERS.join(", ")
        ));
    }
    if model.trim().is_empty() {
        return Err("Modell darf nicht leer sein".into());
    }
    let mut store = load();
    store.models.insert(provider.to_string(), model.to_string());
    save(&store)
}

pub fn get_model(provider: &str) -> Option<String> {
    load().models.get(provider).cloned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub name: String,
    pub has_key: bool,
    pub has_model: bool,
    pub is_default: bool,
}

#[allow(dead_code)] // wired in Phase D (Wizard)
pub fn set_vault_path(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Vault-Pfad darf nicht leer sein".into());
    }
    let mut store = load();
    store.vault_path = Some(trimmed.to_string());
    save(&store)
}

pub fn get_vault_path() -> Option<String> {
    load().vault_path
}

#[allow(dead_code)] // wired in Phase D (Wizard) für „Vault entfernen"
pub fn clear_vault_path() -> Result<(), String> {
    let mut store = load();
    store.vault_path = None;
    save(&store)
}

pub fn list_providers_with_status() -> Vec<ProviderStatus> {
    let store = load();
    let default = store.default_provider.as_deref();
    VALID_PROVIDERS
        .iter()
        .map(|p| ProviderStatus {
            name: (*p).to_string(),
            has_key: store.keys.contains_key(*p),
            has_model: store.models.contains_key(*p),
            is_default: default == Some(*p),
        })
        .collect()
}
