use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const VALID_PROVIDERS: &[&str] = &["claude", "gemini", "zai", "ollama"];

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
}

fn store_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".nexus").join("keys.json")
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
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("Permissions konnten nicht gesetzt werden: {e}"))?;
    Ok(())
}

pub fn set_key(provider: &str, value: &str) -> Result<(), String> {
    if !VALID_PROVIDERS.contains(&provider) {
        return Err(format!("Unbekannter Provider: {provider}. Erlaubt: claude, gemini, zai"));
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
