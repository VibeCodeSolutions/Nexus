use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::BrainDumpEntry;
use super::{Classification, LlmProvider, ProjectSuggestion, SYSTEM_PROMPT, PROJECT_SUGGEST_PROMPT};

pub struct GeminiProvider {
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn categorize_and_summarize(&self, text: &str) -> Result<Classification, String> {
        let prompt = format!("{SYSTEM_PROMPT}\n\nText: {text}");

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Gemini API Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Gemini API {status}: {body}"));
        }

        let gemini_resp: GeminiResponse = response
            .json()
            .await
            .map_err(|e| format!("Gemini Response-Parse Fehler: {e}"))?;

        let raw_text = gemini_resp.candidates.first()
            .and_then(|c| c.content.parts.first())
            .ok_or("Keine Antwort von Gemini")?
            .text.clone();

        // Gemini wraps JSON in markdown code blocks sometimes
        let cleaned = raw_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned)
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw_text}"))
    }

    async fn suggest_projects(&self, entries: &[BrainDumpEntry]) -> Result<Vec<ProjectSuggestion>, String> {
        let entries_text: Vec<String> = entries.iter().map(|e| {
            format!("ID: {}\nText: {}\nKategorie: {}\nSummary: {}", e.id, e.raw_text, e.category, e.summary.as_deref().unwrap_or("-"))
        }).collect();

        let prompt = format!("{PROJECT_SUGGEST_PROMPT}\n\n{}", entries_text.join("\n\n---\n\n"));

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Gemini API Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Gemini API {status}: {body}"));
        }

        let gemini_resp: GeminiResponse = response
            .json()
            .await
            .map_err(|e| format!("Gemini Response-Parse Fehler: {e}"))?;

        let raw_text = gemini_resp.candidates.first()
            .and_then(|c| c.content.parts.first())
            .ok_or("Keine Antwort von Gemini")?
            .text.clone();

        let cleaned = raw_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned)
            .map_err(|e| format!("JSON-Parse Fehler: {e}\nRaw: {raw_text}"))
    }
}
