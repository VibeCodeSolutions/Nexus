use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::keystore::{self, OAuthTokens};
use crate::models::BrainDumpEntry;
use crate::oauth;
use super::{Classification, LlmProvider, ProjectSuggestion, SYSTEM_PROMPT, PROJECT_SUGGEST_PROMPT};

pub enum Auth {
    ApiKey(String),
    OAuth(RwLock<OAuthTokens>),
}

pub struct ClaudeProvider {
    auth: Auth,
    client: Client,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

impl ClaudeProvider {
    pub fn with_api_key(key: String) -> Self {
        Self { auth: Auth::ApiKey(key), client: Client::new() }
    }

    pub fn with_oauth(tokens: OAuthTokens) -> Self {
        Self { auth: Auth::OAuth(RwLock::new(tokens)), client: Client::new() }
    }

    /// Stellt sicher, dass OAuth-Token gültig ist. Refresht bei Bedarf.
    async fn ensure_fresh(&self) -> Result<(), String> {
        let Auth::OAuth(lock) = &self.auth else { return Ok(()); };

        let needs_refresh = {
            let t = lock.read().await;
            // 60s Sicherheitspuffer
            chrono::Utc::now().timestamp() >= t.expires_at - 60
        };

        if needs_refresh {
            let refresh_token = lock.read().await.refresh_token.clone();
            let new_tokens = oauth::refresh(&refresh_token).await?;
            keystore::set_oauth("claude", &new_tokens)?;
            *lock.write().await = new_tokens;
        }
        Ok(())
    }

    async fn build_request(&self, body: &ClaudeRequest) -> Result<RequestBuilder, String> {
        self.ensure_fresh().await?;
        let mut req = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("anthropic-version", "2023-06-01")
            .json(body);

        match &self.auth {
            Auth::ApiKey(k) => {
                req = req.header("x-api-key", k);
            }
            Auth::OAuth(lock) => {
                let t = lock.read().await;
                req = req
                    .header("authorization", format!("Bearer {}", t.access_token))
                    .header("anthropic-beta", "oauth-2025-04-20");
            }
        }
        Ok(req)
    }

    async fn call(&self, body: ClaudeRequest) -> Result<String, String> {
        let req = self.build_request(&body).await?;
        let response = req.send().await
            .map_err(|e| format!("Claude API Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("Claude API {status}: {text}"));
        }

        let parsed: ClaudeResponse = response.json().await
            .map_err(|e| format!("Claude Response-Parse Fehler: {e}"))?;
        Ok(parsed.content.first()
            .ok_or("Keine Antwort von Claude")?
            .text.clone())
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let raw = self.call(ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 256,
            system: SYSTEM_PROMPT.to_string(),
            messages: vec![Message { role: "user".into(), content: text.into() }],
        }).await?;

        serde_json::from_str(&raw)
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }

    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        let entries_text: Vec<String> = entries.iter().map(|e| {
            format!("ID: {}\nText: {}\nKategorie: {}\nSummary: {}",
                e.id, e.raw_text, e.category, e.summary.as_deref().unwrap_or("-"))
        }).collect();

        let raw = self.call(ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 1024,
            system: PROJECT_SUGGEST_PROMPT.to_string(),
            messages: vec![Message { role: "user".into(), content: entries_text.join("\n\n---\n\n") }],
        }).await?;

        serde_json::from_str(&raw)
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }
}
