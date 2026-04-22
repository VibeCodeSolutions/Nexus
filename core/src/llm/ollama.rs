use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::BrainDumpEntry;
use super::{Classification, LlmProvider, ProjectSuggestion, SYSTEM_PROMPT, PROJECT_SUGGEST_PROMPT};

const ENDPOINT: &str = "http://localhost:11434/api/chat";

pub struct OllamaProvider {
    model: String,
    client: Client,
}

impl OllamaProvider {
    pub fn new(model: String) -> Self {
        Self { model, client: Client::new() }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

async fn chat(client: &Client, model: &str, system: &str, user: &str) -> Result<String, String> {
    let req = ChatRequest {
        model: model.to_string(),
        messages: vec![
            Message { role: "system".to_string(), content: system.to_string() },
            Message { role: "user".to_string(), content: user.to_string() },
        ],
        stream: false,
    };

    let resp = client
        .post(ENDPOINT)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("Ollama nicht erreichbar: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama {status}: {text}"));
    }

    let cr: ChatResponse = resp.json().await
        .map_err(|e| format!("Ollama Response Parse Fehler: {e}"))?;

    Ok(cr.message.content)
}

fn extract_json(text: &str) -> &str {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text.trim()
}

fn extract_json_array(text: &str) -> &str {
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return &text[start..=end];
        }
    }
    text.trim()
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let raw = chat(&self.client, &self.model, SYSTEM_PROMPT, text).await?;
        let json = extract_json(&raw);
        serde_json::from_str(json)
            .map_err(|e| format!("JSON Parse Fehler: {e} — Antwort: {raw}"))
    }

    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        let entries_text = entries.iter()
            .map(|e| format!("ID: {}\nText: {}", e.id, e.raw_text))
            .collect::<Vec<_>>()
            .join("\n\n");

        let raw = chat(&self.client, &self.model, PROJECT_SUGGEST_PROMPT, &entries_text).await?;
        let json = extract_json_array(&raw);
        serde_json::from_str(json)
            .map_err(|e| format!("JSON Parse Fehler: {e} — Antwort: {raw}"))
    }
}
