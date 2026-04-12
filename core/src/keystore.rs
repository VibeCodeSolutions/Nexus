use keyring::Entry;

const SERVICE: &str = "nexus";
const VALID_PROVIDERS: &[&str] = &["claude", "gemini"];

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

/// Get any key from the keychain without provider validation.
pub fn get_key_raw(key: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE, key)
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.get_password()
        .map_err(|e| format!("Key '{key}' nicht gefunden: {e}"))
}

pub fn get_key(provider: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE, provider)
        .map_err(|e| format!("Keyring-Fehler: {e}"))?;
    entry.get_password()
        .map_err(|e| format!("Key für '{provider}' nicht gefunden: {e}"))
}
