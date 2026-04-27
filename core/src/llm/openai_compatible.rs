use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::BrainDumpEntry;
use super::{Classification, LlmProvider, ProjectSuggestion, SYSTEM_PROMPT, PROJECT_SUGGEST_PROMPT};

pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    async fn complete(&self, system: &str, user: String) -> Result<String, String> {
        let request = ChatRequest {
            model: &self.model,
            messages: vec![
                Message { role: "system", content: system.to_string() },
                Message { role: "user", content: user },
            ],
            temperature: 0.0,
        };

        let response = self.client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("OpenAI-kompatible API Fehler ({}): {e}", self.base_url))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                401 | 403 => "Bad API Key oder keine Berechtigung — prüfe `nexus-core set-key <provider> <key>`.",
                429 => "Rate limit erreicht — bitte warte kurz und versuche es erneut.",
                500..=599 => "Upstream-Server-Fehler — Provider nicht erreichbar.",
                _ => "Unerwartete Antwort vom Provider.",
            };
            return Err(format!(
                "OpenAI-kompatible API {status} ({}): {hint}\nBody: {body}",
                self.base_url
            ));
        }

        let resp: ChatResponse = response.json().await
            .map_err(|e| format!("OpenAI-kompatible Response-Parse Fehler: {e}"))?;

        resp.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Keine Antwort vom OpenAI-kompatiblen Provider".to_string())
    }
}

use super::clean_json;

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let user = format!("Text: {text}");
        let raw = self.complete(SYSTEM_PROMPT, user).await?;
        serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }

    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        let entries_text: Vec<String> = entries.iter().map(|e| {
            format!(
                "ID: {}\nText: {}\nKategorie: {}\nSummary: {}",
                e.id,
                e.raw_text,
                e.category,
                e.summary.as_deref().unwrap_or("-")
            )
        }).collect();
        let user = entries_text.join("\n\n---\n\n");
        let raw = self.complete(PROJECT_SUGGEST_PROMPT, user).await?;
        serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }
}
