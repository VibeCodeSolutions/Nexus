pub mod claude;
pub mod gemini;

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
    let api_key = keystore::get_key(provider_name)?;

    match provider_name {
        "claude" => Ok(Box::new(claude::ClaudeProvider::new(api_key))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(api_key))),
        _ => Err(format!("Unbekannter Provider: {provider_name}")),
    }
}
