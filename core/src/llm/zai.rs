use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::BrainDumpEntry;
use super::{Classification, LlmProvider, ProjectSuggestion, SYSTEM_PROMPT, PROJECT_SUGGEST_PROMPT};

const ENDPOINT: &str = "https://api.z.ai/api/paas/v4/chat/completions";
const MODEL: &str = "glm-4.6";

pub struct ZaiProvider {
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
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

impl ZaiProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key, client: Client::new() }
    }

    async fn complete(&self, prompt: String) -> Result<String, String> {
        let request = ChatRequest {
            model: MODEL,
            messages: vec![Message { role: "user", content: prompt }],
        };

        let response = self.client
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("z.ai API Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("z.ai API {status}: {body}"));
        }

        let resp: ChatResponse = response.json().await
            .map_err(|e| format!("z.ai Response-Parse Fehler: {e}"))?;

        resp.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Keine Antwort von z.ai".to_string())
    }
}

use super::clean_json;

#[async_trait]
impl LlmProvider for ZaiProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let prompt = format!("{SYSTEM_PROMPT}\n\nText: {text}");
        let raw = self.complete(prompt).await?;
        serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }

    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        let entries_text: Vec<String> = entries.iter().map(|e| {
            format!("ID: {}\nText: {}\nKategorie: {}\nSummary: {}", e.id, e.raw_text, e.category, e.summary.as_deref().unwrap_or("-"))
        }).collect();
        let prompt = format!("{PROJECT_SUGGEST_PROMPT}\n\n{}", entries_text.join("\n\n---\n\n"));
        let raw = self.complete(prompt).await?;
        serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw}"))
    }
}
