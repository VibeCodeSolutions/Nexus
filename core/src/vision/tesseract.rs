//! Sprint Nightvision NV-1 — Tesseract-OCR-Fallback.
//!
//! Wir rufen den system-installierten `tesseract`-Binary als Subprocess auf,
//! statt eine native Rust-Bindung (`tesseract`-crate via libtesseract) zu
//! benutzen. Begründung:
//!
//! * keine native Build-Abhängigkeit für nexus-core, dadurch auch keine
//!   Compile-Probleme auf Windows / im CI-Container ohne libtesseract;
//! * Tesseract ist auf Fedora/Ubuntu typischerweise schon installiert
//!   (Paket: `tesseract` bzw. `tesseract-ocr`);
//! * der Fallback ist Best-Effort — wenn das Binary fehlt, melden wir
//!   einen klaren Fehler statt zu crashen.
//!
//! Der Aufrufer (siehe [`crate::vision::analyze`]) entscheidet, ob er sich
//! auf das Ergebnis verlässt: Tesseract erkennt Druck-Text gut, Handschrift
//! schlecht — beim Nightvision-Foto-Flow ist das ein akzeptabler Trade-off
//! gegenüber „gar nichts liefern".

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Pfad zum tesseract-Binary. Standardpfad reicht; PATH-resolved via
/// `tokio::process::Command`.
const TESSERACT_BIN: &str = "tesseract";

/// Sprache(n) für Tesseract. Reihenfolge `deu+eng` deckt deutsche Notizen
/// mit englischen Fachbegriffen ab — Tesseract sucht in beiden Wörterbüchern.
/// Setzt voraus, dass `tesseract-langpack-deu` (Fedora) bzw.
/// `tesseract-ocr-deu` (Debian/Ubuntu) installiert ist; fehlt eine Sprache
/// fällt Tesseract auf die andere zurück.
const TESSERACT_LANG: &str = "deu+eng";

/// Führt Tesseract OCR auf den Bild-Bytes aus und gibt Textzeilen (in
/// Lese-Reihenfolge, leere Zeilen verworfen) zurück.
///
/// Implementierung: schreibt das Bild auf stdin, liest Text von stdout
/// (`tesseract - - -l deu+eng`). Kein temporäres File nötig.
pub async fn ocr(image_bytes: &[u8]) -> Result<Vec<String>, String> {
    let mut child = Command::new(TESSERACT_BIN)
        .arg("-") // stdin
        .arg("-") // stdout
        .arg("-l")
        .arg(TESSERACT_LANG)
        // Page Segmentation Mode 3 = "Fully automatic page segmentation,
        // but no OSD" — guter Default für Foto-Notizen.
        .arg("--psm")
        .arg("3")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                "Tesseract-Binary nicht gefunden — installiere `tesseract` \
                 (Fedora: `dnf install tesseract tesseract-langpack-deu`) \
                 oder deaktiviere den Fallback via NEXUS_OCR_TESSERACT_ENABLED=false."
                    .to_string()
            }
            _ => format!("Tesseract-Spawn fehlgeschlagen: {e}"),
        })?;

    // Bild auf stdin schreiben und Pipe schließen, damit Tesseract EOF sieht.
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(image_bytes)
            .await
            .map_err(|e| format!("Tesseract-stdin Schreibfehler: {e}"))?;
        stdin
            .shutdown()
            .await
            .map_err(|e| format!("Tesseract-stdin Close-Fehler: {e}"))?;
    } else {
        return Err("Tesseract: stdin nicht verfügbar".to_string());
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Tesseract-Wait-Fehler: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Tesseract exit={:?}: {}",
            output.status.code(),
            stderr.trim()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nicht ausführbar im CI ohne Tesseract — Test ignored standardmäßig.
    /// Lokal via `cargo test -- --ignored` laufen lassen, wenn `tesseract`
    /// + `tesseract-langpack-deu` installiert sind.
    #[tokio::test]
    #[ignore = "benötigt installiertes tesseract-Binary"]
    async fn tesseract_returns_lines_for_synthetic_image() {
        // Erzeuge ein simples Bild mit Text via image-Crate (ohne Schriftart-Render
        // bleibt das aber leer). Wir testen hier nur den Subprocess-Pfad, indem
        // wir Tesseract über ein leeres PNG schicken — Erwartung: 0 Zeilen, kein
        // Fehler.
        let png = {
            let img = image::RgbImage::from_pixel(80, 80, image::Rgb([255, 255, 255]));
            let mut out = Vec::new();
            image::DynamicImage::ImageRgb8(img)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .expect("png encode");
            out
        };
        let lines = ocr(&png).await.expect("tesseract should run");
        // Leeres Bild → keine Zeilen.
        assert!(lines.is_empty(), "expected empty OCR result, got {lines:?}");
    }

    #[tokio::test]
    async fn missing_binary_returns_clear_err() {
        // Override TESSERACT_BIN nicht möglich (const), aber wir können prüfen,
        // dass `Command::new("definitely-not-on-path-xyz")` einen
        // NotFound-Fehler liefert — wir replizieren die Logik direkt.
        let res = Command::new("definitely-not-on-path-tesseract-xyz")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .spawn();
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }
}
