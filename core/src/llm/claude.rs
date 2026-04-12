use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{Classification, LlmProvider, SYSTEM_PROMPT};

pub struct ClaudeProvider {
    api_key: String,
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
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 256,
            system: SYSTEM_PROMPT.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: text.to_string(),
            }],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Claude API Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Claude API {status}: {body}"));
        }

        let claude_resp: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| format!("Claude Response-Parse Fehler: {e}"))?;

        let raw_text = claude_resp.content.first()
            .ok_or("Keine Antwort von Claude")?
            .text.clone();

        serde_json::from_str(&raw_text)
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw_text}"))
    }
}
