// Einige Symbole (resize::PreparedImage::mime-Konstante, VisionError-Varianten
// im Test-Pfad) werden erst durch den HTTP-Handler in NV-2 voll konsumiert —
// daher lokale Dead-Code-Toleranz. Das `#[allow(dead_code)]` sitzt jetzt am
// `mod vision;` in `main.rs` (redundantes Inner-Attribut hier entfernt, sonst
// löst es `clippy::duplicated_attributes` aus).

//! Sprint Nightvision NV-1 — Vision-LLM + OCR-Fallback für Foto-Sparks.
//!
//! Pipeline (siehe `docs/sprints/nightvision-photo-ocr.md`):
//!
//! 1. Bild → [`prepare_image`]: dekodieren, auf max 1920px Längskante
//!    runterrechnen, als JPEG re-enkodieren.
//! 2. [`analyze`] ruft entweder einen [`VisionProvider`] (`groq` etc.) auf,
//!    oder fällt auf [`tesseract::ocr`] + bestehenden [`LlmProvider::categorize_and_summarize`](crate::llm::LlmProvider::categorize_and_summarize)
//!    zurück, wenn Vision deaktiviert ist oder fehlschlägt.
//!
//! Die Pipeline gibt strukturierten [`VisionAnalysis`]-Output zurück:
//! geordnete OCR-Zeilen + Tag-Vorschläge, die der HTTP-Endpoint (NV-2)
//! als SSE-Frames an den Client streamt.

pub mod claude;
pub mod gemini;
pub mod groq;
pub mod ollama;
pub mod openai_compatible;
pub mod resize;
pub mod tesseract;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::VisionConfig;
use crate::keystore;

/// Ergebnis einer Bild-Analyse: extrahierte Textzeilen (in Lese-Reihenfolge)
/// und vorgeschlagene Tags. Beide Listen können leer sein — z. B. wenn ein
/// Bild keinen sinnvollen Text enthält. Der Aufrufer entscheidet dann, ob
/// trotzdem ein Spark angelegt wird (mit leerem `raw_text`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisionAnalysis {
    pub text_lines: Vec<String>,
    pub suggested_tags: Vec<String>,
}

/// Provider-agnostisches Interface für vision-fähige LLMs. Analog zum
/// bestehenden [`LlmProvider`](crate::llm::LlmProvider), aber bewusst getrennt,
/// weil Vision-Endpoints einen anderen Request-Body brauchen (Bild-Bytes
/// statt purem Text) und nicht jeder LLM-Provider Vision unterstützt.
#[async_trait]
pub trait VisionProvider: Send + Sync {
    /// Analysiert `image_bytes` (bereits via [`prepare_image`] vorbereitet)
    /// und liefert Text + Tags. `mime` ist der Mime-Type der Bytes
    /// (z. B. `image/jpeg`) — relevant, weil manche Provider den Mime-Type
    /// im Data-URI explizit erwarten.
    async fn analyze_image(
        &self,
        image_bytes: &[u8],
        mime: &str,
    ) -> Result<VisionAnalysis, String>;
}

/// Fehler-Klassen der Analyse-Pipeline. Wird vom HTTP-Endpoint (NV-2) auf
/// HTTP-Statuscodes gemappt: `Disabled` → 503, `BadInput` → 400, sonst 502.
#[derive(Debug)]
pub enum VisionError {
    /// `vision.enabled = false` — User-Toggle aus den Settings.
    Disabled,
    /// Bild konnte nicht dekodiert / wiedergegeben werden.
    BadInput(String),
    /// Weder Vision-Provider noch Tesseract liefern Resultate.
    AllFailed { vision: Option<String>, fallback: Option<String> },
}

impl std::fmt::Display for VisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisionError::Disabled => write!(
                f,
                "Vision-Analyse ist deaktiviert (NEXUS_VISION_ENABLED=false)."
            ),
            VisionError::BadInput(msg) => write!(f, "Bild-Eingabe ungültig: {msg}"),
            VisionError::AllFailed { vision, fallback } => write!(
                f,
                "Vision + Tesseract-Fallback fehlgeschlagen — vision: {}, fallback: {}",
                vision.as_deref().unwrap_or("n/a"),
                fallback.as_deref().unwrap_or("n/a"),
            ),
        }
    }
}

impl std::error::Error for VisionError {}

/// Vollständige Analyse-Pipeline. Reihenfolge:
///
/// 1. Vision-Provider versuchen (laut `cfg.provider`), wenn `cfg.enabled`.
/// 2. Bei Fehler / kein Provider → Tesseract-Fallback (wenn `cfg.tesseract_enabled`).
/// 3. Tesseract-Output durch [`tag_text`] für Tag-Vorschläge via bestehendem
///    `LlmProvider` (default-provider aus der Hauptkonfig).
///
/// Production-Wrapper um [`analyze_with_provider`]: lädt den Vision-Provider
/// aus der Config (Keystore-Read). Für Tests existiert [`analyze_with_provider`]
/// als Direkt-Einstieg mit injizierbarem `&dyn VisionProvider` (NV1-001-VOL).
pub async fn analyze(
    cfg: &VisionConfig,
    image_bytes: &[u8],
    mime: &str,
    default_llm_provider: &str,
) -> Result<VisionAnalysis, VisionError> {
    if !cfg.enabled {
        return Err(VisionError::Disabled);
    }

    match create_vision_provider(cfg) {
        Ok(provider) => {
            analyze_with_provider(
                Some(provider.as_ref()),
                None,
                cfg.tesseract_enabled,
                image_bytes,
                mime,
                default_llm_provider,
            )
            .await
        }
        Err(reason) => {
            analyze_with_provider(
                None,
                Some(reason),
                cfg.tesseract_enabled,
                image_bytes,
                mime,
                default_llm_provider,
            )
            .await
        }
    }
}

/// Trait-Injizierbarer Einstieg für Tests + interne Wiederverwendung. Wenn
/// `provider` `Some` ist, läuft der Vision-Pfad zuerst; sonst (`None`) wird
/// `provider_unavailable_reason` als Vision-Fehler dokumentiert und direkt
/// in den Tesseract-Fallback gesprungen.
pub async fn analyze_with_provider(
    provider: Option<&dyn VisionProvider>,
    provider_unavailable_reason: Option<String>,
    tesseract_enabled: bool,
    image_bytes: &[u8],
    mime: &str,
    default_llm_provider: &str,
) -> Result<VisionAnalysis, VisionError> {
    // 1. Vision-Provider — sofern verfügbar.
    let vision_err: Option<String> = match provider {
        Some(p) => match p.analyze_image(image_bytes, mime).await {
            Ok(analysis) if !analysis.text_lines.is_empty() => return Ok(analysis),
            Ok(empty) => {
                // Erfolgreich aber leer — kein Sinn, jetzt Tesseract zu probieren,
                // weil das Bild offenbar keinen Text hat. Wir geben das leere
                // Result zurück (Aufrufer entscheidet).
                return Ok(empty);
            }
            Err(e) => Some(e),
        },
        None => provider_unavailable_reason,
    };

    // 2. Tesseract-Fallback.
    if tesseract_enabled {
        match tesseract::ocr(image_bytes).await {
            Ok(text_lines) if !text_lines.is_empty() => {
                // 3. Tag-Generation über den Standard-LLM-Provider.
                let suggested_tags = tag_text(&text_lines, default_llm_provider)
                    .await
                    .unwrap_or_default();
                return Ok(VisionAnalysis {
                    text_lines,
                    suggested_tags,
                });
            }
            Ok(_empty) => {
                return Err(VisionError::AllFailed {
                    vision: vision_err,
                    fallback: Some("Tesseract liefert keinen Text".to_string()),
                });
            }
            Err(e) => {
                return Err(VisionError::AllFailed {
                    vision: vision_err,
                    fallback: Some(e),
                });
            }
        }
    }

    Err(VisionError::AllFailed {
        vision: vision_err,
        fallback: Some("Tesseract-Fallback deaktiviert".to_string()),
    })
}

/// Factory für den konfigurierten Vision-Provider. Holt den API-Key aus
/// dem Keystore (analog [`crate::llm::create_provider`]).
pub fn create_vision_provider(cfg: &VisionConfig) -> Result<Box<dyn VisionProvider>, String> {
    match cfg.provider.as_str() {
        "groq" => {
            let key = keystore::get_key("groq").map_err(|e| {
                format!("Vision-Provider `groq`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .unwrap_or_else(|| "meta-llama/llama-4-scout-17b-16e-instruct".to_string());
            Ok(Box::new(groq::GroqVisionProvider::new(model, key)))
        }
        "ollama" => {
            // Ollama läuft lokal — kein API-Key, Endpoint via OLLAMA_HOST (default
            // http://localhost:11434). Modell aus cfg.model bzw. cfg.vision_model
            // (Default `llava`).
            let provider = ollama::OllamaVisionProvider::new(cfg)?;
            Ok(Box::new(provider))
        }
        "claude" => {
            let key = keystore::get_key("claude").map_err(|e| {
                format!("Vision-Provider `claude`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "claude-haiku-4-5-20251001".to_string());
            Ok(Box::new(claude::ClaudeVisionProvider::new(model, key)))
        }
        "gemini" => {
            let key = keystore::get_key("gemini").map_err(|e| {
                format!("Vision-Provider `gemini`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "gemini-2.5-flash".to_string());
            Ok(Box::new(gemini::GeminiVisionProvider::new(model, key)))
        }
        "openai" => {
            let key = keystore::get_key("openai").map_err(|e| {
                format!("Vision-Provider `openai`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "gpt-4o-mini".to_string());
            Ok(Box::new(openai_compatible::OpenAiCompatibleVisionProvider::new(
                "https://api.openai.com/v1/chat/completions",
                model,
                key,
            )))
        }
        "openrouter" => {
            let key = keystore::get_key("openrouter").map_err(|e| {
                format!("Vision-Provider `openrouter`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "anthropic/claude-haiku-4.5".to_string());
            Ok(Box::new(openai_compatible::OpenAiCompatibleVisionProvider::new(
                "https://openrouter.ai/api/v1/chat/completions",
                model,
                key,
            )))
        }
        "xai" => {
            let key = keystore::get_key("xai").map_err(|e| {
                format!("Vision-Provider `xai`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "grok-2-vision-1212".to_string());
            Ok(Box::new(openai_compatible::OpenAiCompatibleVisionProvider::new(
                "https://api.x.ai/v1/chat/completions",
                model,
                key,
            )))
        }
        "mistral" => {
            let key = keystore::get_key("mistral").map_err(|e| {
                format!("Vision-Provider `mistral`: kein API-Key konfiguriert ({e})")
            })?;
            let model = cfg
                .model
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "pixtral-12b-2409".to_string());
            Ok(Box::new(openai_compatible::OpenAiCompatibleVisionProvider::new(
                "https://api.mistral.ai/v1/chat/completions",
                model,
                key,
            )))
        }
        other => Err(format!(
            "Vision-Provider `{other}` ist (noch) nicht implementiert. \
             Aktuell verfügbar: groq, ollama, claude, gemini, openai, openrouter, xai, mistral."
        )),
    }
}

/// Schiebt einen OCR-Output durch den bestehenden Text-LLM-Provider, um
/// Tag-Vorschläge zu erhalten. Wird nur im Tesseract-Fallback-Pfad benötigt.
async fn tag_text(lines: &[String], llm_provider_name: &str) -> Result<Vec<String>, String> {
    let provider = crate::llm::create_provider(llm_provider_name)?;
    let joined = lines.join("\n");
    let classification = provider.categorize_and_summarize(&joined).await?;
    Ok(classification.tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_config_returns_disabled_error() {
        let cfg = VisionConfig {
            enabled: false,
            provider: "groq".to_string(),
            model: None,
            vision_model: "llava".to_string(),
            tesseract_enabled: false,
        };
        let err = analyze(&cfg, b"irrelevant", "image/jpeg", "noop")
            .await
            .expect_err("disabled config must short-circuit");
        assert!(matches!(err, VisionError::Disabled));
    }

    #[test]
    fn vision_error_displays_human_readable() {
        let e = VisionError::AllFailed {
            vision: Some("groq 401".into()),
            fallback: Some("tesseract missing".into()),
        };
        let s = format!("{e}");
        assert!(s.contains("groq 401"));
        assert!(s.contains("tesseract missing"));
    }

    // ---- Mock-VisionProvider (NV1-001-VOL Auflage) -----------------------------

    /// Test-Stub mit injizierbarer Antwort. Wenn `result` `Some` ist, liefert
    /// `analyze_image` diese Analyse; ist `result` `None`, kommt ein Error.
    struct MockVisionProvider {
        result: Option<VisionAnalysis>,
        error_msg: Option<String>,
    }

    #[async_trait]
    impl VisionProvider for MockVisionProvider {
        async fn analyze_image(
            &self,
            _image_bytes: &[u8],
            _mime: &str,
        ) -> Result<VisionAnalysis, String> {
            if let Some(a) = &self.result {
                Ok(a.clone())
            } else {
                Err(self
                    .error_msg
                    .clone()
                    .unwrap_or_else(|| "mock vision failure".into()))
            }
        }
    }

    #[tokio::test]
    async fn mock_pipeline_happy_path() {
        let mock = MockVisionProvider {
            result: Some(VisionAnalysis {
                text_lines: vec!["Sprint Planning".into(), "Mustafa → USB-Stick".into()],
                suggested_tags: vec!["BRAIN DUMP".into(), "MEETING".into()],
            }),
            error_msg: None,
        };
        let out = analyze_with_provider(
            Some(&mock),
            None,
            /*tesseract_enabled=*/ false,
            b"dummy-bytes",
            "image/jpeg",
            "noop",
        )
        .await
        .expect("vision-pfad muss erfolgreich liefern");
        assert_eq!(out.text_lines.len(), 2);
        assert_eq!(out.suggested_tags[0], "BRAIN DUMP");
    }

    #[tokio::test]
    async fn mock_provider_failure_without_tesseract_returns_allfailed() {
        let mock = MockVisionProvider {
            result: None,
            error_msg: Some("mock 503".into()),
        };
        let err = analyze_with_provider(
            Some(&mock),
            None,
            /*tesseract_enabled=*/ false,
            b"dummy",
            "image/jpeg",
            "noop",
        )
        .await
        .expect_err("ohne tesseract muss provider-fail propagieren");
        match err {
            VisionError::AllFailed { vision, fallback } => {
                assert_eq!(vision.as_deref(), Some("mock 503"));
                assert!(fallback.as_deref().unwrap().contains("deaktiviert"));
            }
            other => panic!("unerwarteter error: {other}"),
        }
    }

    #[tokio::test]
    async fn missing_provider_reports_unavailable_reason() {
        let err = analyze_with_provider(
            None,
            Some("kein API-Key konfiguriert".into()),
            /*tesseract_enabled=*/ false,
            b"dummy",
            "image/jpeg",
            "noop",
        )
        .await
        .expect_err("kein provider + kein fallback → AllFailed");
        match err {
            VisionError::AllFailed { vision, .. } => {
                assert_eq!(vision.as_deref(), Some("kein API-Key konfiguriert"));
            }
            other => panic!("unerwarteter error: {other}"),
        }
    }

    #[tokio::test]
    async fn mock_provider_empty_result_short_circuits() {
        // Ein Bild ohne Text liefert leere Listen — Pipeline soll *nicht*
        // den Tesseract-Fallback bemühen, sondern das leere Ergebnis durchreichen.
        let mock = MockVisionProvider {
            result: Some(VisionAnalysis::default()),
            error_msg: None,
        };
        let out = analyze_with_provider(
            Some(&mock),
            None,
            /*tesseract_enabled=*/ true,
            b"dummy",
            "image/jpeg",
            "noop",
        )
        .await
        .expect("leerer vision-erfolg propagiert");
        assert!(out.text_lines.is_empty());
        assert!(out.suggested_tags.is_empty());
    }
}
