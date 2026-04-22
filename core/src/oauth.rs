use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::keystore::OAuthTokens;

// Anthropic OAuth (Claude Pro/Max Subscription)
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
const SCOPES: &str = "org:create_api_key user:profile user:inference";

pub struct PkcePair {
    pub verifier: String,
    pub challenge: String,
}

pub fn generate_pkce() -> PkcePair {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    PkcePair { verifier, challenge }
}

pub fn random_state() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn build_authorize_url(pkce: &PkcePair, state: &str) -> String {
    format!(
        "{AUTHORIZE_URL}?client_id={cid}&response_type=code&redirect_uri={ru}&scope={sc}&code_challenge={cc}&code_challenge_method=S256&state={st}",
        cid = urlencoding::encode(CLIENT_ID),
        ru = urlencoding::encode(REDIRECT_URI),
        sc = urlencoding::encode(SCOPES),
        cc = urlencoding::encode(&pkce.challenge),
        st = urlencoding::encode(state),
    )
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

/// Tauscht den vom User gepasteten Code (Format: "code#state" oder nur "code") gegen Tokens.
pub async fn exchange_code(
    raw_code: &str,
    verifier: &str,
    state: &str,
) -> Result<OAuthTokens, String> {
    let (code, paste_state) = match raw_code.split_once('#') {
        Some((c, s)) => (c.trim().to_string(), Some(s.trim().to_string())),
        None => (raw_code.trim().to_string(), None),
    };

    if let Some(s) = &paste_state {
        if s != state {
            return Err("State stimmt nicht überein — Auth abgebrochen.".to_string());
        }
    }

    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code.as_str()),
        ("redirect_uri", REDIRECT_URI),
        ("client_id", CLIENT_ID),
        ("code_verifier", verifier),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token-Exchange Netzwerkfehler: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Token-Exchange {status}: {text}"));
    }

    let tr: TokenResponse = resp.json().await
        .map_err(|e| format!("Token-Response Parse Fehler: {e}"))?;

    Ok(OAuthTokens {
        access_token: tr.access_token,
        refresh_token: tr.refresh_token,
        expires_at: chrono::Utc::now().timestamp() + tr.expires_in,
    })
}

pub async fn refresh(refresh_token: &str) -> Result<OAuthTokens, String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
    ];

    let resp = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh Netzwerkfehler: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Refresh {status}: {text}"));
    }

    let tr: TokenResponse = resp.json().await
        .map_err(|e| format!("Refresh-Response Parse Fehler: {e}"))?;

    Ok(OAuthTokens {
        access_token: tr.access_token,
        refresh_token: tr.refresh_token,
        expires_at: chrono::Utc::now().timestamp() + tr.expires_in,
    })
}
