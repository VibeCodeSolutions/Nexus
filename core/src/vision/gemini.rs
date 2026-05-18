//! Sprint S24-VISION-FIX P3 — Google Gemini als Vision-Provider.
//!
//! Gemini API: `POST https://generativelanguage.googleapis.com/v1beta/models/<model>:generateContent?key=<key>`
//! Bild-Encoding als `inline_data` mit `mime_type` + `data` (base64).
//!
//! Response-Layout: `{"candidates":[{"content":{"parts":[{"text":"…"}]}}]}` —
//! analog zum bestehenden Text-LLM-Provider in `crate::llm::gemini`.

use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{VisionAnalysis, VisionProvider};

const GEMINI_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

const VISION_PROMPT: &str = r#"Du bist ein OCR- und Tagging-Assistent für ein persönliches Notiz-System.
Analysiere das Bild und extrahiere ALLE lesbaren Texte in Lese-Reihenfolge.
Liste pro Eintrag eine Textzeile.
Schlage zusätzlich 3–6 prägnante Tags vor (in GROSSBUCHSTABEN, deutsch),
die den Inhalt thematisch zusammenfassen.

Antworte AUSSCHLIESSLICH mit validem JSON in genau diesem Schema:
{
  "text_lines": ["<Zeile 1>", "<Zeile 2>", ...],
  "tags": ["<TAG1>", "<TAG2>", ...]
}
Keine Markdown-Blöcke, kein zusätzlicher Text — nur das JSON-Objekt."#;

pub struct GeminiVisionProvider {
    model: String,
    api_key: String,
    client: Client,
}

impl GeminiVisionProvider {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    contents: Vec<Content<'a>>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part<'a> {
    Text {
        text: &'a str,
    },
    Inline {
        inline_data: InlineData<'a>,
    },
}

#[derive(Serialize)]
struct InlineData<'a> {
    mime_type: &'a str,
    data: String,
}

#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    #[serde(default)]
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    #[serde(default)]
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize)]
struct VisionJson {
    #[serde(default)]
    text_lines: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

fn clean_json(raw: &str) -> &str {
    let trimmed = raw.trim();
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_end_matches("```")
        .trim();
    if let (Some(start), Some(end)) = (stripped.find('{'), stripped.rfind('}')) {
        if end >= start {
            return &stripped[start..=end];
        }
    }
    stripped
}

#[async_trait]
impl VisionProvider for GeminiVisionProvider {
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        mime: &str,
    ) -> Result<VisionAnalysis, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

        let request = GenerateRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text { text: VISION_PROMPT },
                    Part::Inline {
                        inline_data: InlineData {
                            mime_type: mime,
                            data: b64,
                        },
                    },
                ],
            }],
        };

        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_BASE, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Gemini-Vision Request-Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                400 => {
                    "Ungueltige Anfrage — moeglicherweise inkompatibler mime-type oder Bild zu gross."
                }
                401 | 403 => {
                    "Bad API Key oder keine Berechtigung — pruefe `nexus-core set-key gemini <key>`."
                }
                404 => {
                    "Modell nicht gefunden — Vision-Modell-Name aktuell? \
                     (NEXUS_VISION_MODEL ueberschreibt den Default `gemini-2.5-flash`.)"
                }
                413 => "Bild zu gross fuer den Provider — Resize-Step hat versagt?",
                429 => "Rate Limit / Quota erreicht — Tesseract-Fallback greift, sofern aktiviert.",
                500..=599 => "Gemini-Server-Fehler — spaeter erneut versuchen.",
                _ => "Unerwartete Gemini-Antwort.",
            };
            return Err(format!("Gemini-Vision {status}: {hint}\nBody: {body}"));
        }

        let resp: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Gemini-Vision Response-Parse-Fehler: {e}"))?;

        let raw = resp
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content)
            .and_then(|c| c.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| "Gemini-Vision: keine Candidates im Response".to_string())?;

        let parsed: VisionJson = serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("Gemini-Vision JSON-Parse-Fehler: {e}\nRaw: {raw}"))?;

        let mut seen = std::collections::HashSet::new();
        let tags: Vec<String> = parsed
            .tags
            .into_iter()
            .map(|t| t.trim().to_uppercase())
            .filter(|t| !t.is_empty())
            .filter(|t| seen.insert(t.clone()))
            .collect();

        let text_lines: Vec<String> = parsed
            .text_lines
            .into_iter()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        Ok(VisionAnalysis {
            text_lines,
            suggested_tags: tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_json_strips_markdown_fences() {
        assert_eq!(clean_json("```json\n{\"a\": 1}\n```"), "{\"a\": 1}");
        assert_eq!(clean_json("```\n{\"a\":1}```"), "{\"a\":1}");
    }

    #[test]
    fn clean_json_extracts_object_from_prose() {
        let raw = "Hier: {\"text_lines\":[\"x\"],\"tags\":[\"Y\"]} — Ende.";
        assert_eq!(
            clean_json(raw),
            "{\"text_lines\":[\"x\"],\"tags\":[\"Y\"]}"
        );
    }

    #[test]
    fn clean_json_passthrough_when_no_fences() {
        let raw = "{\"text_lines\":[],\"tags\":[]}";
        assert_eq!(clean_json(raw), raw);
    }

    #[test]
    fn new_constructs_provider_with_model_and_key() {
        let provider = GeminiVisionProvider::new("gemini-2.5-flash", "g-test");
        assert_eq!(provider.model, "gemini-2.5-flash");
        assert_eq!(provider.api_key, "g-test");
    }
}
