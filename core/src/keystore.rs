use keyring::Entry;
use serde::{Deserialize, Serialize};

const SERVICE: &str = "nexus";
const VALID_PROVIDERS: &[&str] = &["claude", "gemini", "zai"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Unix-Sekunden, wann access_token abläuft
    pub expires_at: i64,
}

pub fn set_key(provider: &str, value: &str) -> Result<(), String> {
    if !VALID_PROVIDERS.contains(&provider) {
        return Err(format!("Unbekannter Provider: {provider}. Erlaubt: claude, gemini"));
    }

    let entry = Entry::new(SERVICE, provider)
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.set_password(value)
        .map_err(|e| format!("Key konnte nicht gespeichert werden: {e}"))?;

    Ok(())
}

pub fn get_key(provider: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE, provider)
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.get_password()
        .map_err(|e| format!("Key für '{provider}' nicht gefunden: {e}"))
}

pub fn delete_key(provider: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, provider)
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.delete_credential()
        .map_err(|e| format!("Key konnte nicht gelöscht werden: {e}"))
}

fn oauth_account(provider: &str) -> String {
    format!("{provider}_oauth")
}

pub fn set_oauth(provider: &str, tokens: &OAuthTokens) -> Result<(), String> {
    let json = serde_json::to_string(tokens)
        .map_err(|e| format!("Serialisierung fehlgeschlagen: {e}"))?;
    let entry = Entry::new(SERVICE, &oauth_account(provider))
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.set_password(&json)
        .map_err(|e| format!("OAuth-Token konnte nicht gespeichert werden: {e}"))
}

pub fn get_oauth(provider: &str) -> Result<OAuthTokens, String> {
    let entry = Entry::new(SERVICE, &oauth_account(provider))
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    let json = entry.get_password()
        .map_err(|e| format!("OAuth-Token für '{provider}' nicht gefunden: {e}"))?;
    serde_json::from_str(&json)
        .map_err(|e| format!("OAuth-Token-Parse Fehler: {e}"))
}

pub fn delete_oauth(provider: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE, &oauth_account(provider))
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.delete_credential()
        .map_err(|e| format!("OAuth-Token konnte nicht gelöscht werden: {e}"))
}
