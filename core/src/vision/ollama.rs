//! Sprint S24-VISION-FIX SM-S24-RE-2 — Ollama-Vision-Provider (lokal, z.B. llava).
//!
//! Ollama läuft default auf `http://localhost:11434` und akzeptiert für
//! Vision-Modelle (z.B. `llava`) den Endpoint `POST /api/generate` mit
//! einem `images: ["<base64>"]`-Array. Das Modell antwortet als reiner
//! Text — kein JSON-Format-Forcing wie bei Groq —, daher parsen wir den
//! Output über die zwei Marker `TEXT:` und `TAGS:`.
//!
//! Vorteil dieses Providers: kein API-Key, läuft komplett offline, sobald
//! das Modell einmal gepullt wurde (`ollama pull llava`).

use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{VisionAnalysis, VisionProvider};
use crate::config::VisionConfig;

/// Default-Endpoint, analog zum bestehenden Ollama-LLM-Provider
/// (`crate::llm::ollama::ENDPOINT`). Anders als der Chat-Endpoint
/// nutzen Vision-Modelle `/api/generate`.
const OLLAMA_GENERATE_ENDPOINT_PATH: &str = "/api/generate";
const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434";

const VISION_PROMPT: &str = r#"Du bekommst ein Foto. Extrahiere ALLEN sichtbaren Text aus dem Bild (auch Handschrift) in Lese-Reihenfolge, Zeile für Zeile. Schlage danach 2-4 prägnante deutsche Tags vor.

Antworte EXAKT in diesem Format:

TEXT:
<Zeile 1>
<Zeile 2>
...

TAGS:
<tag1>, <tag2>, <tag3>"#;

pub struct OllamaVisionProvider {
    base_url: String,
    model: String,
    client: Client,
}

impl OllamaVisionProvider {
    /// Erzeugt einen neuen Ollama-Vision-Provider aus der Vision-Config.
    /// `base_url` kommt aus `OLLAMA_HOST` (env), Fallback `http://localhost:11434`.
    /// `model` aus `cfg.model` (NEXUS_VISION_MODEL) oder `cfg.vision_model`
    /// (default `llava`).
    pub fn new(cfg: &VisionConfig) -> Result<Self, String> {
        let base_url = std::env::var("OLLAMA_HOST")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| OLLAMA_DEFAULT_BASE_URL.to_string());
        let model = cfg
            .model
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| cfg.vision_model.clone());
        if model.trim().is_empty() {
            return Err(
                "Vision-Provider `ollama`: kein Modell konfiguriert (NEXUS_VISION_MODEL leer)."
                    .to_string(),
            );
        }
        Ok(Self {
            base_url,
            model,
            client: Client::new(),
        })
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    images: Vec<String>,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
}

/// Parser für das `TEXT:` / `TAGS:` Antwortformat. Robust gegen:
/// - Variationen mit/ohne Doppelpunkt nach den Markern
/// - Leerzeilen zwischen Marker und Inhalt
/// - Fehlender TAGS-Abschnitt (→ leeres Tag-Array)
/// - Fehlender TEXT-Abschnitt (→ leeres text_lines-Array)
fn parse_vision_response(raw: &str) -> VisionAnalysis {
    let trimmed = raw.trim();
    // Marker-Indizes case-insensitive suchen.
    let lower = trimmed.to_lowercase();

    let text_marker = lower.find("text:").or_else(|| lower.find("text\n"));
    let tags_marker = lower.find("tags:").or_else(|| lower.find("tags\n"));

    let (text_section, tags_section): (&str, &str) = match (text_marker, tags_marker) {
        (Some(t), Some(g)) if g > t => {
            // TEXT-Block: nach `TEXT:` bis vor `TAGS:`. Wir schneiden auf
            // den originalen `trimmed` (gleiche Längen, da to_lowercase
            // pro Byte arbeitet).
            let text_start = t + "text:".len();
            (&trimmed[text_start..g], &trimmed[g + "tags:".len()..])
        }
        (Some(t), None) => {
            let text_start = t + "text:".len();
            (&trimmed[text_start..], "")
        }
        (None, Some(g)) => ("", &trimmed[g + "tags:".len()..]),
        _ => {
            // Keine Marker — gesamtes Output als Text behandeln, keine Tags.
            (trimmed, "")
        }
    };

    let text_lines: Vec<String> = text_section
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    // Tags: erste nicht-leere Zeile nach `TAGS:` → komma-separiert.
    let tags_line = tags_section
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let mut seen = std::collections::HashSet::new();
    let suggested_tags: Vec<String> = tags_line
        .split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .filter(|t| seen.insert(t.clone()))
        .collect();

    VisionAnalysis {
        text_lines,
        suggested_tags,
    }
}

#[async_trait]
impl VisionProvider for OllamaVisionProvider {
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        _mime: &str,
    ) -> Result<VisionAnalysis, String> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

        let request = GenerateRequest {
            model: &self.model,
            prompt: VISION_PROMPT,
            images: vec![b64],
            stream: false,
        };

        let url = format!("{}{}", self.base_url, OLLAMA_GENERATE_ENDPOINT_PATH);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Ollama-Vision nicht erreichbar ({url}): {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let hint = match status.as_u16() {
                404 => "Modell nicht installiert? Versuche `ollama pull llava`.",
                500..=599 => "Ollama-Server-Fehler — läuft `ollama serve`?",
                _ => "Unerwartete Ollama-Antwort.",
            };
            return Err(format!("Ollama-Vision {status}: {hint}\nBody: {body}"));
        }

        let resp: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Ollama-Vision Response-Parse-Fehler: {e}"))?;

        Ok(parse_vision_response(&resp.response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_response_extracts_text_and_tags() {
        let raw = "TEXT:\nSprint Planning\nMustafa -> USB-Stick\nDemo Freitag\n\nTAGS:\nmeeting, sprint, usb";
        let out = parse_vision_response(raw);
        assert_eq!(out.text_lines.len(), 3);
        assert_eq!(out.text_lines[0], "Sprint Planning");
        assert_eq!(out.text_lines[2], "Demo Freitag");
        assert_eq!(out.suggested_tags, vec!["meeting", "sprint", "usb"]);
    }

    #[test]
    fn parse_handles_missing_tags_section() {
        let raw = "TEXT:\nNur eine Zeile";
        let out = parse_vision_response(raw);
        assert_eq!(out.text_lines, vec!["Nur eine Zeile"]);
        assert!(out.suggested_tags.is_empty());
    }

    #[test]
    fn parse_empty_response_yields_empty_analysis() {
        let out = parse_vision_response("");
        assert!(out.text_lines.is_empty());
        assert!(out.suggested_tags.is_empty());
    }

    #[test]
    fn parse_dedupes_tags_and_trims_whitespace() {
        let raw = "TEXT:\nZeile 1\n\nTAGS:\n meeting , meeting , sprint ";
        let out = parse_vision_response(raw);
        assert_eq!(out.suggested_tags, vec!["meeting", "sprint"]);
    }

    #[test]
    fn parse_handles_no_markers_treats_as_text_only() {
        // Modell folgt dem Format nicht — wir geben den Gesamt-Output als
        // Text-Zeilen zurück, damit User wenigstens den OCR-Output sieht.
        let raw = "Eine Zeile\nNoch eine";
        let out = parse_vision_response(raw);
        assert_eq!(out.text_lines, vec!["Eine Zeile", "Noch eine"]);
        assert!(out.suggested_tags.is_empty());
    }

    #[test]
    fn new_uses_vision_model_default_when_model_unset() {
        let cfg = VisionConfig {
            enabled: true,
            provider: "ollama".into(),
            model: None,
            vision_model: "llava".into(),
            tesseract_enabled: false,
        };
        let provider = OllamaVisionProvider::new(&cfg).expect("ollama provider");
        assert_eq!(provider.model, "llava");
    }

    #[test]
    fn new_prefers_explicit_model_over_vision_model() {
        let cfg = VisionConfig {
            enabled: true,
            provider: "ollama".into(),
            model: Some("llava:13b".into()),
            vision_model: "llava".into(),
            tesseract_enabled: false,
        };
        let provider = OllamaVisionProvider::new(&cfg).expect("ollama provider");
        assert_eq!(provider.model, "llava:13b");
    }
}
