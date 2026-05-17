//! Sprint Nightvision NV-1 — Bild-Vorverarbeitung für den Foto-Braindump.
//!
//! Vision-LLMs sind in Input-Size limitiert (Groq Llama-Vision z. B. ~4 MB
//! Base64-Payload pro Request) und teurer/langsamer mit großen Bildern. Wir
//! skalieren deshalb jedes Bild serverseitig auf max. 1920 px Längskante und
//! re-enkodieren als JPEG-Q85. Das bringt typische Smartphone-Fotos (~12 MP)
//! von 4-6 MB auf 200-400 KB ohne sichtbaren Qualitätsverlust für OCR.

use image::{ImageFormat, ImageReader};
use std::io::Cursor;

/// Maximale Längskante des Outputs in Pixeln. Wert ist Konvention; ggf.
/// pro Modell anpassbar, wenn Provider strenger limitieren.
pub const MAX_DIM: u32 = 1920;

/// JPEG-Qualität für das Re-Encoding (1-100). 85 ist Konsens-Default für
/// „visuell verlustfrei" bei deutlich kleinerer Dateigröße als 95+.
pub const JPEG_QUALITY: u8 = 85;

/// Output von [`prepare_image`]: re-enkodierte JPEG-Bytes + Mime-Type.
/// Mime ist immer `image/jpeg`, wird aber explizit zurückgegeben, damit
/// die Vision-Provider-API den Wert weitergeben können ohne im Aufrufer
/// hardgecoded zu sein.
#[derive(Debug, Clone)]
pub struct PreparedImage {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

/// Dekodiert beliebige Bild-Bytes, skaliert auf max [`MAX_DIM`] px Längskante
/// (nur wenn größer — Upscaling wäre sinnfrei), und re-enkodiert als JPEG.
///
/// Fehler-Pfade:
/// - Bild-Format wird nicht erkannt → `Err(...)`
/// - Encoder schlägt fehl → `Err(...)`
/// Beides sollte der Aufrufer als `400 Bad Request` durchreichen.
pub fn prepare_image(input: &[u8]) -> Result<PreparedImage, String> {
    let reader = ImageReader::new(Cursor::new(input))
        .with_guessed_format()
        .map_err(|e| format!("Bild-Format konnte nicht erkannt werden: {e}"))?;

    let img = reader
        .decode()
        .map_err(|e| format!("Bild konnte nicht dekodiert werden: {e}"))?;

    let (w, h) = (img.width(), img.height());
    let scaled = if w.max(h) > MAX_DIM {
        // image::imageops::FilterType::Triangle ist guter Kompromiss aus
        // Geschwindigkeit und Schärfe; Lanczos3 wäre besser aber spürbar
        // langsamer und für OCR-Input nicht wichtig.
        img.resize(MAX_DIM, MAX_DIM, image::imageops::FilterType::Triangle)
    } else {
        img
    };

    let mut out = Vec::with_capacity(input.len() / 4);
    // Konvertiere RGBA → RGB falls nötig, da JPEG keinen Alpha-Kanal kennt.
    // `.to_rgb8()` rendert eine RGB-Kopie unabhängig vom Input-ColorType.
    let rgb = scaled.to_rgb8();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    rgb.write_with_encoder(encoder)
        .map_err(|e| format!("JPEG-Encoding fehlgeschlagen: {e}"))?;

    // Belt-and-suspenders: leerer Output ist illegal — Decoder hätte schon
    // gemeckert, aber wir wollen einen klaren Fehler statt eines 0-byte-Posts
    // an den LLM-Provider.
    if out.is_empty() {
        return Err("JPEG-Encoder lieferte 0 Bytes".to_string());
    }

    // Touch ImageFormat so der Import in Cargo nicht als unused gewarnt wird,
    // falls wir später dynamic-Format-Output dranbauen wollen. Reine Doku.
    let _ = ImageFormat::Jpeg;

    Ok(PreparedImage {
        bytes: out,
        mime: "image/jpeg",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Erzeugt ein gültiges, einfaches PNG-Bild im Speicher als Test-Input.
    fn synth_png(width: u32, height: u32) -> Vec<u8> {
        let buf = image::RgbImage::from_fn(width, height, |x, _y| {
            image::Rgb([((x * 255 / width.max(1)) as u8), 128, 64])
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(buf)
            .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
            .expect("encode test png");
        out
    }

    #[test]
    fn small_image_passes_through_dimensions() {
        let png = synth_png(640, 480);
        let prepared = prepare_image(&png).expect("prepare ok");
        // Re-decode um Dimensionen zu prüfen.
        let img = ImageReader::new(Cursor::new(&prepared.bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!(img.width(), 640);
        assert_eq!(img.height(), 480);
        assert_eq!(prepared.mime, "image/jpeg");
    }

    #[test]
    fn large_image_is_downscaled_to_max_dim() {
        let png = synth_png(4000, 3000);
        let prepared = prepare_image(&png).expect("prepare ok");
        let img = ImageReader::new(Cursor::new(&prepared.bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert!(img.width().max(img.height()) <= MAX_DIM);
        // Aspect ratio soll grob erhalten bleiben (4:3 → längere Kante = 1920).
        assert_eq!(img.width(), MAX_DIM);
    }

    #[test]
    fn invalid_input_returns_err() {
        let res = prepare_image(b"definitely not an image");
        assert!(res.is_err());
    }
}
