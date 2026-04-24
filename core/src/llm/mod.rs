pub mod claude;
pub mod gemini;
pub mod ollama;
pub mod openai_compatible;
pub mod zai;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::keystore;
use crate::models::BrainDumpEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub category: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSuggestion {
    pub name: String,
    pub description: String,
    pub braindump_ids: Vec<String>,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String>;
    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String>;
}

pub const SYSTEM_PROMPT: &str = r#"Du bist ein Kategorisierungs-Assistent für ein persönliches Notiz-System.
Analysiere den folgenden Text und antworte AUSSCHLIESSLICH mit validem JSON in diesem Format:
{
  "category": "<eine von: Idea, Task, Worry, Question, Random>",
  "summary": "<kurze Zusammenfassung in einem Satz>",
  "tags": ["<relevante Tags>"]
}
Keine zusätzliche Erklärung, nur das JSON."#;

pub const PROJECT_SUGGEST_PROMPT: &str = r#"Du bist ein Projekt-Planungs-Assistent. Analysiere die folgenden BrainDump-Einträge und schlage sinnvolle Projekt-Gruppierungen vor.
Fasse thematisch zusammengehörige Einträge zu Projekten zusammen.
Antworte AUSSCHLIESSLICH mit validem JSON in diesem Format:
[
  {
    "name": "<Projektname>",
    "description": "<kurze Beschreibung des Projekts>",
    "braindump_ids": ["<id1>", "<id2>"]
  }
]
Nur Einträge gruppieren, die wirklich zusammengehören. Nicht jeder Eintrag muss einem Projekt zugeordnet werden.
Keine zusätzliche Erklärung, nur das JSON-Array."#;

pub struct NoOpProvider;

#[async_trait]
impl LlmProvider for NoOpProvider {
    async fn categorize_and_summarize(&self, _text: &str) -> Result<Classification, String> {
        Err("Kein LLM-Provider konfiguriert. Nutze: nexus set-key claude <key>".to_string())
    }

    async fn suggest_projects(&self, _entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        Err("Kein LLM-Provider konfiguriert. Nutze: nexus set-key claude <key>".to_string())
    }
}

pub fn create_provider(provider_name: &str) -> Result<Box<dyn LlmProvider>, String> {
    match provider_name {
        "claude" => {
            // OAuth zuerst, dann API-Key als Fallback
            if let Ok(tokens) = keystore::get_oauth("claude") {
                return Ok(Box::new(claude::ClaudeProvider::with_oauth(tokens)));
            }
            let api_key = keystore::get_key("claude")?;
            Ok(Box::new(claude::ClaudeProvider::with_api_key(api_key)))
        }
        "gemini" => {
            let api_key = keystore::get_key("gemini")?;
            Ok(Box::new(gemini::GeminiProvider::new(api_key)))
        }
        "zai" => {
            let api_key = keystore::get_key("zai")?;
            Ok(Box::new(zai::ZaiProvider::new(api_key)))
        }
        "ollama" => {
            let model = keystore::get_key("ollama").unwrap_or_else(|_| "qwen2.5:3b".to_string());
            Ok(Box::new(ollama::OllamaProvider::new(model)))
        }
        "openai" => {
            let key = keystore::get_key("openai")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://api.openai.com/v1/chat/completions",
                "gpt-4o-mini",
                key,
            )))
        }
        "mistral" => {
            let key = keystore::get_key("mistral")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://api.mistral.ai/v1/chat/completions",
                "mistral-small-latest",
                key,
            )))
        }
        "groq" => {
            let key = keystore::get_key("groq")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://api.groq.com/openai/v1/chat/completions",
                "llama-3.1-70b-versatile",
                key,
            )))
        }
        "deepseek" => {
            let key = keystore::get_key("deepseek")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://api.deepseek.com/v1/chat/completions",
                "deepseek-chat",
                key,
            )))
        }
        "openrouter" => {
            let key = keystore::get_key("openrouter")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://openrouter.ai/api/v1/chat/completions",
                "openai/gpt-4o-mini",
                key,
            )))
        }
        _ => Err(format!("Unbekannter Provider: {provider_name}")),
    }
}
