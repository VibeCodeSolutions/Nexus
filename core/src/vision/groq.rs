//! Sprint Nightvision NV-1 — Groq-Vision-Provider (Llama-4-Scout / Llama-3.2-Vision).
//!
//! Groq's Chat-Completions-Endpoint nimmt das OpenAI-Multimodal-Format an:
//! `messages[0].content` ist ein Array aus `text`- und `image_url`-Items,
//! wobei `image_url.url` ein `data:image/jpeg;base64,…`-URI sein darf. Der
//! Provider gibt eine Standard-Chat-Completion zurück; wir lassen das Modell
//! per System-Prompt strukturiertes JSON liefern (`text_lines` + `tags`).

use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{VisionAnalysis, VisionProvider};

const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";

const VISION_SYSTEM_PROMPT: &str = r#"Du bist ein OCR- und Tagging-Assistent für ein persönliches Notiz-System.
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

pub struct GroqVisionProvider {
    model: String,
    api_key: String,
    client: Client,
}

impl GroqVisionProvider {
    pub fn new(model: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
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
    /// Groq akzeptiert `response_format: { "type": "json_object" }`; wenn das
    /// Modell es unterstützt erzwingt es valides JSON. Falls nicht unterstützt
    /// ignoriert Groq das Feld stillschweigend — kein Fehler.
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
    // Manche Modelle schreiben Prosa vor dem JSON-Objekt; auf erstes `{`
    // und letztes `}` schneiden, falls vorhanden.
    if let (Some(start), Some(end)) = (stripped.find('{'), stripped.rfind('}')) {
        if end >= start {
            return &stripped[start..=end];
        }
    }
    stripped
}

#[async_trait]
impl VisionProvider for GroqVisionProvider {
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        mime: &str,
    ) -> Result<VisionAnalysis, String> {
        // Data-URI bauen: `data:image/jpeg;base64,...`. Spec erlaubt URL-safe
        // Standard-Base64; reqwest setzt den Body als JSON, wir müssen den
        // String selbst encoden.
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
        let data_uri = format!("data:{mime};base64,{b64}");

        let user_msg = RequestMessage {
            role: "user",
            content: vec![
                ContentItem::Text {
                    text: VISION_SYSTEM_PROMPT,
                },
                ContentItem::ImageUrl {
                    image_url: ImageUrl { url: data_uri },
                },
            ],
        };

        let request = ChatRequest {
            model: &self.model,
            messages: vec![user_msg],
            temperature: 0.0,
            response_format: ResponseFormat { kind: "json_object" },
        };

        let response = self
            .client
            .post(GROQ_ENDPOINT)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Groq-Vision Request-Fehler: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                401 | 403 => {
                    "Bad API Key oder keine Berechtigung — prüfe `nexus-core set-key groq <key>`."
                }
                404 => {
                    "Modell-Endpoint nicht gefunden — Vision-Modell-Name aktuell? \
                     (NEXUS_VISION_MODEL überschreibt den Default.)"
                }
                413 => "Bild zu groß für den Provider — Resize-Step hat versagt?",
                429 => "Rate Limit / Quota erreicht — Tesseract-Fallback greift, sofern aktiviert.",
                500..=599 => "Groq-Server-Fehler — später erneut versuchen.",
                _ => "Unerwartete Groq-Antwort.",
            };
            return Err(format!(
                "Groq-Vision {status}: {hint}\nBody: {body}"
            ));
        }

        let resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Groq-Vision Response-Parse-Fehler: {e}"))?;

        let raw = resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Groq-Vision: keine Choices im Response".to_string())?;

        let parsed: VisionJson = serde_json::from_str(clean_json(&raw))
            .map_err(|e| format!("Groq-Vision JSON-Parse-Fehler: {e}\nRaw: {raw}"))?;

        // Tags normalisieren: trim, leere weg, in Großbuchstaben (Modell hält
        // sich nicht immer dran). Duplikate eliminieren bei Beibehaltung der
        // Reihenfolge.
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
        assert_eq!(
            clean_json("```json\n{\"a\": 1}\n```"),
            "{\"a\": 1}"
        );
        assert_eq!(clean_json("```\n{\"a\":1}```"), "{\"a\":1}");
    }

    #[test]
    fn clean_json_extracts_object_from_prose() {
        let raw = "Hier ist das Ergebnis: {\"text_lines\":[],\"tags\":[]} — Fertig.";
        assert_eq!(clean_json(raw), "{\"text_lines\":[],\"tags\":[]}");
    }
}
