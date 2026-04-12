pub mod claude;
pub mod gemini;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::keystore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub category: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String>;
}

pub const SYSTEM_PROMPT: &str = r#"Du bist ein Kategorisierungs-Assistent für ein persönliches Notiz-System.
Analysiere den folgenden Text und antworte AUSSCHLIESSLICH mit validem JSON in diesem Format:
{
  "category": "<eine von: Idea, Task, Worry, Question, Random>",
  "summary": "<kurze Zusammenfassung in einem Satz>",
  "tags": ["<relevante Tags>"]
}
Keine zusätzliche Erklärung, nur das JSON."#;

pub fn create_provider(provider_name: &str) -> Result<Box<dyn LlmProvider>, String> {
    let api_key = keystore::get_key(provider_name)?;

    match provider_name {
        "claude" => Ok(Box::new(claude::ClaudeProvider::new(api_key))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(api_key))),
        _ => Err(format!("Unbekannter Provider: {provider_name}")),
    }
}
