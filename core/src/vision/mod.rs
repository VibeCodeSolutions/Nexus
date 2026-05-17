// NV-1 verdrahtet noch keine HTTP-Routen — das passiert in NV-2. Bis dahin
// ist das gesamte Vision-Modul „API ohne Konsumenten", weshalb wir die
// Dead-Code-Warnings auf Modul-Ebene unterdrücken.
#![allow(dead_code, unused_assignments)]

//! Sprint Nightvision NV-1 — Vision-LLM + OCR-Fallback für Foto-Braindumps.
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

pub mod groq;
pub mod resize;
pub mod tesseract;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::VisionConfig;
use crate::keystore;

/// Ergebnis einer Bild-Analyse: extrahierte Textzeilen (in Lese-Reihenfolge)
/// und vorgeschlagene Tags. Beide Listen können leer sein — z. B. wenn ein
/// Bild keinen sinnvollen Text enthält. Der Aufrufer entscheidet dann, ob
/// trotzdem ein Braindump angelegt wird (mit leerem `raw_text`).
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
/// Der `default_llm_provider`-Param ist der Name des Text-LLMs, das für die
/// Tag-Generation bei Tesseract-Output benutzt wird (z. B. `cfg.default_provider`
/// aus der Hauptkonfig). Bei reinem Vision-Pfad ungenutzt.
pub async fn analyze(
    cfg: &VisionConfig,
    image_bytes: &[u8],
    mime: &str,
    default_llm_provider: &str,
) -> Result<VisionAnalysis, VisionError> {
    if !cfg.enabled {
        return Err(VisionError::Disabled);
    }

    let mut vision_err: Option<String> = None;

    // 1. Vision-Provider.
    match create_vision_provider(cfg) {
        Ok(provider) => match provider.analyze_image(image_bytes, mime).await {
            Ok(analysis) if !analysis.text_lines.is_empty() => return Ok(analysis),
            Ok(empty) => {
                // Erfolgreich aber leer — kein Sinn, jetzt Tesseract zu probieren,
                // weil das Bild offenbar keinen Text hat. Wir geben das leere
                // Result zurück (Aufrufer entscheidet).
                return Ok(empty);
            }
            Err(e) => vision_err = Some(e),
        },
        Err(e) => vision_err = Some(e),
    }

    // 2. Tesseract-Fallback.
    if cfg.tesseract_enabled {
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
        other => Err(format!(
            "Vision-Provider `{other}` ist (noch) nicht implementiert. \
             Aktuell verfügbar: groq."
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
}
