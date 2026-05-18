//! Sprint S24-VISION-FIX P4 — Generischer OpenAI-kompatibler Vision-Adapter.
//!
//! Ein einziger Provider-Struct fuer vier Backends, die alle das
//! OpenAI-Chat-Completions-Format mit Vision-Content beherrschen:
//!
//! - `openai`     → `https://api.openai.com/v1/chat/completions`, `gpt-4o-mini`
//! - `openrouter` → `https://openrouter.ai/api/v1/chat/completions`, `anthropic/claude-haiku-4.5`
//! - `xai`        → `https://api.x.ai/v1/chat/completions`, `grok-2-vision-1212`
//! - `mistral`    → `https://api.mistral.ai/v1/chat/completions`, `pixtral-12b-2409`
//!
//! Body-Format (Vision-Content):
//! ```json
//! {
//!   "model": "...",
//!   "messages": [{
//!     "role": "user",
//!     "content": [
//!       {"type":"text","text":"<prompt>"},
//!       {"type":"image_url","image_url":{"url":"data:<mime>;base64,<b64>"}}
//!     ]
//!   }],
//!   "temperature": 0.0,
//!   "response_format": {"type":"json_object"}
//! }
//! ```
//! `response_format` wird gesetzt — Provider, die es nicht kennen,
//! ignorieren es i.d.R. stillschweigend; `clean_json` fangen wir trotzdem
//! ab, falls Prosa oder Fences geliefert werden.

use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{VisionAnalysis, VisionProvider};

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

pub struct OpenAiCompatibleVisionProvider {
    base_url: String,
    model: String,
    api_key: String,
    client: Client,
}

impl OpenAiCompatibleVisionProvider {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<RequestMessage<'a>>,
    temperature: f32,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct RequestMessage<'a> {
    role: &'a str,
    content: Vec<ContentItem<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentItem<'a> {
    #[serde(rename = "text")]
    Text { text: &'a str },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
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
impl VisionProvider for OpenAiCompatibleVisionProvider {
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        mime: &str,
    ) -> Result<VisionAnalysis, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
        let data_uri = format!("data:{mime};base64,{b64}");

        let user_msg = RequestMessage {
            role: "user",
            content: vec![
                ContentItem::Text { text: VISION_PROMPT },
                ContentItem::ImageUrl {
                    image_url: ImageUrl { url: data_uri },
                },
            ],
        };

        let request = ChatRequest {
            model: &self.model,
            messages: vec![user_msg],
            temperature: 0.0,
            response_format: ResponseFormat {
                kind: "json_object",
            },
        };

        let response = self
            .client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                format!(
                    "OpenAI-kompatible Vision Request-Fehler ({}): {e}",
                    self.base_url
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                400 => {
                    "Ungueltige Anfrage — Modell vielleicht nicht vision-faehig oder Bild zu gross."
                }
                401 | 403 => {
                    "Bad API Key oder keine Berechtigung — pruefe `nexus-core set-key <provider> <key>`."
                }
                404 => {
                    "Modell/Endpoint nicht gefunden — Vision-Modell-Name aktuell? \
                     (NEXUS_VISION_MODEL ueberschreibt den Default.)"
                }
                413 => "Bild zu gross fuer den Provider — Resize-Step hat versagt?",
                429 => "Rate Limit / Quota erreicht — Tesseract-Fallback greift, sofern aktiviert.",
                500..=599 => "Upstream-Server-Fehler — spaeter erneut versuchen.",
                _ => "Unerwartete Antwort vom Provider.",
            };
            return Err(format!(
                "OpenAI-kompatible Vision {status} ({}): {hint}\nBody: {body}",
                self.base_url
            ));
        }

        let resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("OpenAI-kompatible Vision Response-Parse-Fehler: {e}"))?;

        let raw = resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| {
                "OpenAI-kompatible Vision: keine Choices im Response".to_string()
            })?;

        let parsed: VisionJson = serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("OpenAI-kompatible Vision JSON-Parse-Fehler: {e}\nRaw: {raw}"))?;

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
        let raw = "Antwort: {\"text_lines\":[],\"tags\":[]}.";
        assert_eq!(clean_json(raw), "{\"text_lines\":[],\"tags\":[]}");
    }

    #[test]
    fn clean_json_passthrough_when_no_fences() {
        let raw = "{\"text_lines\":[\"x\"],\"tags\":[\"Y\"]}";
        assert_eq!(clean_json(raw), raw);
    }

    #[test]
    fn new_constructs_provider_with_all_fields() {
        let provider = OpenAiCompatibleVisionProvider::new(
            "https://api.openai.com/v1/chat/completions",
            "gpt-4o-mini",
            "sk-test",
        );
        assert_eq!(provider.base_url, "https://api.openai.com/v1/chat/completions");
        assert_eq!(provider.model, "gpt-4o-mini");
        assert_eq!(provider.api_key, "sk-test");
    }
}
