pub mod claude;
pub mod gemini;
pub mod obsidian;
pub mod ollama;
pub mod openai_compatible;
pub mod zai;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::keystore;
use crate::models::SparkEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub category: String,
    pub summary: String,
    pub tags: Vec<String>,
    /// Obsidian-Briefkasten Pending-Pattern (Phase B):
    /// Wenn `Some`, hat der Provider eine Inbox-Datei im Vault geschrieben
    /// und gibt nur einen Pending-Marker zurück. Der Aufrufer setzt dann
    /// `sparks.classification_status = 'pending'` und persistiert die
    /// `inbox_id` in `sparks.nexus_inbox_id`. Klassische LLM-Provider
    /// lassen das Feld leer (serde-default → `None`).
    #[serde(default)]
    pub inbox_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSuggestion {
    pub name: String,
    pub description: String,
    pub spark_ids: Vec<String>,
    #[serde(default = "default_suggestion_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub reason: Option<String>,
}

fn default_suggestion_confidence() -> f64 { 0.85 }

/// LLM-Vorschlag für eine Verknüpfung zwischen einem source-Knoten und target-Knoten.
/// Wird vom Background-Task `extract_links_for_recent` konsumiert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSuggestion {
    pub target_type: String,    // 'spark' | 'project'
    pub target_id: String,
    #[serde(default = "default_link_relation")]
    pub relation: String,       // 'related' | 'mentions'
    pub confidence: f64,
    pub reason: Option<String>,
}

fn default_link_relation() -> String { "related".to_string() }

/// Knoten-Beschreibung für den LLM-extract_links-Prompt-Kontext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRef {
    pub node_type: String,      // 'spark' | 'project'
    pub id: String,
    pub label: String,          // Spark.summary oder Project.name
}

/// FEAT-001: Ein durch den LLM aus einem Spark extrahiertes Action-Item.
/// Wird in der Tasks-Tabelle als individueller Task persistiert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub title: String,
    #[serde(default = "default_action_priority")]
    pub priority: String,           // 'low' | 'medium' | 'high'
    #[serde(default)]
    pub due_date: Option<String>,   // ISO-8601 (YYYY-MM-DD), optional
    #[serde(default)]
    pub category: Option<String>,   // freier Tag, optional
}

fn default_action_priority() -> String { "medium".to_string() }

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String>;
    async fn suggest_projects(&self, entries: &[SparkEntry]) -> Result<Vec<ProjectSuggestion>, String>;

    /// SM-PR-002: Default-Impl liefert leere Liste — Provider können opt-in overriden.
    /// Pflicht-Override in claude.rs + ollama.rs (zwei Default-Provider). Andere optional.
    async fn extract_links(
        &self,
        _source_text: &str,
        _candidates: &[NodeRef],
    ) -> Result<Vec<LinkSuggestion>, String> {
        Ok(Vec::new())
    }

    /// FEAT-001: Extrahiert einzelne Action-Items (Tasks) aus einem Spark-Text.
    /// Default-Impl liefert leere Liste (analog extract_links). Pflicht-Override
    /// in claude.rs + ollama.rs (zwei Default-Provider).
    async fn extract_action_items(
        &self,
        _text: &str,
    ) -> Result<Vec<ActionItem>, String> {
        Ok(Vec::new())
    }
}

pub const SYSTEM_PROMPT: &str = r#"Du bist ein Kategorisierungs-Assistent für ein persönliches Notiz-System.
Analysiere den folgenden Text und antworte AUSSCHLIESSLICH mit validem JSON in diesem Format:
{
  "category": "<eine von: Idea, Task, Worry, Question, Random>",
  "summary": "<kurze Zusammenfassung in einem Satz>",
  "tags": ["<relevante Tags>"]
}
Keine zusätzliche Erklärung, nur das JSON."#;

pub const PROJECT_SUGGEST_PROMPT: &str = r#"Du bist ein Projekt-Planungs-Assistent. Analysiere die folgenden Spark-Einträge und schlage sinnvolle Projekt-Gruppierungen vor.
Fasse thematisch zusammengehörige Einträge zu Projekten zusammen.
Antworte AUSSCHLIESSLICH mit validem JSON in diesem Format:
[
  {
    "name": "<Projektname>",
    "description": "<kurze Beschreibung des Projekts>",
    "spark_ids": ["<id1>", "<id2>"],
    "confidence": <0.0-1.0>,
    "reason": "<warum diese Gruppierung>"
  }
]
Nur Einträge gruppieren, die wirklich zusammengehören. Nicht jeder Eintrag muss einem Projekt zugeordnet werden.
confidence sollte ehrlich 0.5-1.0 sein, je sicherer du bist desto höher.
Keine zusätzliche Erklärung, nur das JSON-Array."#;

pub const EXTRACT_ACTION_ITEMS_PROMPT: &str = r#"Du analysierst einen Spark-Eintrag (Sprach-Notiz, Gedankendump) und extrahierst daraus konkrete Action-Items, die als Tasks abgearbeitet werden können.
Wenn der Text mehrere Themen enthält ("Kauf Milch, ruf Kai an, Todo-App fixen bis Freitag"), gib für jedes ein Item zurück.
Wenn der Text gar keine umsetzbare Aufgabe enthält (reine Gedanken, Fragen, Sorgen), gib ein leeres Array zurück.
Antworte AUSSCHLIESSLICH mit validem JSON-Array:
[
  {
    "title": "<kurzer, klarer Task-Titel im Imperativ, max 80 Zeichen>",
    "priority": "low" | "medium" | "high",
    "due_date": "<YYYY-MM-DD oder null>",
    "category": "<optionaler Tag wie 'Arbeit', 'Privat', 'Einkauf' oder null>"
  }
]
Regeln:
- title ist Pflicht und immer aussagekräftig (kein "Milch" sondern "Milch kaufen")
- priority ist medium wenn keine Dringlichkeits-Hinweise im Text
- due_date nur setzen wenn explizit ein Datum/Tag genannt ist (z.B. "bis Freitag", "morgen") — sonst null
- category nur wenn klar erkennbarer Kontext — sonst null
Keine zusätzliche Erklärung, nur das JSON-Array."#;

pub const EXTRACT_LINKS_PROMPT: &str = r#"Du analysierst Verknüpfungen zwischen Notizen.
Gegeben ist ein Quell-Text und eine Liste von Kandidaten-Knoten (Sparks/Projekte).
Gib eine Liste von Verknüpfungen zurück, die thematisch sinnvoll sind.
Antworte AUSSCHLIESSLICH mit validem JSON-Array:
[
  {
    "target_type": "spark" | "project",
    "target_id": "<id aus Kandidatenliste>",
    "relation": "related" | "mentions",
    "confidence": <0.0-1.0>,
    "reason": "<warum diese Verknüpfung>"
  }
]
Nur Verknüpfungen, die wirklich zusammenpassen. Lieber wenige hoch-Confidence als viele schwache.
Keine zusätzliche Erklärung, nur das JSON-Array."#;

pub struct NoOpProvider;

#[async_trait]
impl LlmProvider for NoOpProvider {
    async fn categorize_and_summarize(&self, _text: &str) -> Result<Classification, String> {
        Err("Kein LLM-Provider konfiguriert. Nutze: nexus set-key claude <key>".to_string())
    }

    async fn suggest_projects(&self, _entries: &[SparkEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        Err("Kein LLM-Provider konfiguriert. Nutze: nexus set-key claude <key>".to_string())
    }
}

pub fn create_provider(provider_name: &str) -> Result<Box<dyn LlmProvider>, String> {
    match provider_name {
        "noop" => Ok(Box::new(NoOpProvider)),
        "obsidian" => {
            // Vault-Pfad analog zur Config-Logik: env > keystore > Fehler.
            let vault = std::env::var("NEXUS_VAULT_PATH")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .or_else(crate::keystore::get_vault_path)
                .ok_or_else(|| {
                    "Obsidian-Provider: kein Vault-Pfad konfiguriert. \
                     Setze NEXUS_VAULT_PATH oder konfiguriere via Wizard (Phase D)."
                        .to_string()
                })?;
            Ok(Box::new(obsidian::ObsidianProvider::new(
                std::path::PathBuf::from(vault),
            )?))
        }
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
            // Empty/whitespace-only entries fall back to the default model —
            // Ollama's API rejects model="" with 400 "model is required".
            let model = keystore::get_key("ollama")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "qwen2.5:3b".to_string());
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
                "llama-3.3-70b-versatile",
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
        "xai" => {
            let key = keystore::get_key("xai")?;
            Ok(Box::new(openai_compatible::OpenAiCompatibleProvider::new(
                "https://api.x.ai/v1/chat/completions",
                "grok-3-mini",
                key,
            )))
        }
        _ => Err(format!("Unbekannter Provider: {provider_name}")),
    }
}
