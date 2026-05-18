//! Sprint S24-VISION-FIX P2 — Anthropic Claude als Vision-Provider.
//!
//! Anthropic Messages API: `POST https://api.anthropic.com/v1/messages`
//! mit Image-Source `{"type":"base64","media_type":"<mime>","data":"<b64>"}`.
//!
//! Anthropic-Header sind eigenwillig: `x-api-key`, `anthropic-version`,
//! kein Bearer. Response-Layout: `{"content":[{"type":"text","text":"…"}]}`.
//!
//! Claude befolgt JSON-Schema-Anweisungen aus dem Prompt zuverlässig, daher
//! kein `response_format`-Forcing (gibt es bei Anthropic ohnehin nicht
//! direkt) — wir nehmen den `clean_json`-Helper für Robustheit, falls
//! Claude doch mal Prosa drumherum schreibt.

use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{VisionAnalysis, VisionProvider};

const CLAUDE_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

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

pub struct ClaudeVisionProvider {
    model: String,
    api_key: String,
    client: Client,
}

impl ClaudeVisionProvider {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<RequestMessage<'a>>,
}

#[derive(Serialize)]
struct RequestMessage<'a> {
    role: &'a str,
    content: Vec<ContentItem<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentItem<'a> {
    #[serde(rename = "image")]
    Image { source: ImageSource<'a> },
    #[serde(rename = "text")]
    Text { text: &'a str },
}

#[derive(Serialize)]
struct ImageSource<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    media_type: &'a str,
    data: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
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
impl VisionProvider for ClaudeVisionProvider {
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        mime: &str,
    ) -> Result<VisionAnalysis, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

        let user_msg = RequestMessage {
            role: "user",
            content: vec![
                ContentItem::Image {
                    source: ImageSource {
                        kind: "base64",
                        media_type: mime,
                        data: b64,
                    },
                },
                ContentItem::Text { text: VISION_PROMPT },
            ],
        };

        let request = MessagesRequest {
            model: &self.model,
            max_tokens: 1024,
            messages: vec![user_msg],
        };

        let response = self
            .client
            .post(CLAUDE_ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Claude-Vision Request-Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                401 | 403 => {
                    "Bad API Key oder keine Berechtigung — prüfe `nexus-core set-key claude <key>`."
                }
                404 => {
                    "Modell-Endpoint nicht gefunden — Vision-Modell-Name aktuell? \
                     (NEXUS_VISION_MODEL überschreibt den Default.)"
                }
                413 => "Bild zu groß für den Provider — Resize-Step hat versagt?",
                429 => "Rate Limit / Quota erreicht — Tesseract-Fallback greift, sofern aktiviert.",
                500..=599 => "Anthropic-Server-Fehler — später erneut versuchen.",
                _ => "Unerwartete Claude-Antwort.",
            };
            return Err(format!("Claude-Vision {status}: {hint}\nBody: {body}"));
        }

        let resp: MessagesResponse = response
            .json()
            .await
            .map_err(|e| format!("Claude-Vision Response-Parse-Fehler: {e}"))?;

        let raw = resp
            .content
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or_else(|| "Claude-Vision: kein Content-Block im Response".to_string())?;

        let parsed: VisionJson = serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("Claude-Vision JSON-Parse-Fehler: {e}\nRaw: {raw}"))?;

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
        let raw = "Hier ist das Ergebnis: {\"text_lines\":[],\"tags\":[]} — Fertig.";
        assert_eq!(clean_json(raw), "{\"text_lines\":[],\"tags\":[]}");
    }

    #[test]
    fn clean_json_passthrough_when_no_fences() {
        assert_eq!(
            clean_json("{\"text_lines\":[\"a\"],\"tags\":[\"X\"]}"),
            "{\"text_lines\":[\"a\"],\"tags\":[\"X\"]}"
        );
    }

    #[test]
    fn new_constructs_provider_with_model_and_key() {
        let provider = ClaudeVisionProvider::new("claude-haiku-4-5-20251001", "sk-ant-test");
        assert_eq!(provider.model, "claude-haiku-4-5-20251001");
        assert_eq!(provider.api_key, "sk-ant-test");
    }
}
