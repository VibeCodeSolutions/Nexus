use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use crate::config::home_dir;

fn token_path() -> PathBuf {
    home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus_token")
}

/// Generate a random pairing token and store it with restrictive permissions.
pub fn generate_token() -> Result<String, String> {
    use base64::Engine;
    use rand::Rng;

    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let path = token_path();
    // Write with 0600 permissions (owner-only read/write)
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, token.as_bytes()))
        .map_err(|e| format!("Token konnte nicht gespeichert werden: {e}"))?;

    Ok(token)
}

/// Get the existing token, or generate one if none exists.
pub fn get_or_create_token() -> Result<String, String> {
    match fs::read_to_string(token_path()) {
        Ok(token) if !token.trim().is_empty() => Ok(token.trim().to_string()),
        _ => generate_token(),
    }
}

/// Get pairing info as a `nexus://pair` deep-link URI for QR code.
pub fn pairing_uri(bind_addr: &str) -> Result<String, String> {
    let token = get_or_create_token()?;

    let port = bind_addr.split(':').last().unwrap_or("7777");
    let url = match local_ip_address::local_ip() {
        Ok(ip) => format!("http://{}:{}", ip, port),
        Err(_) => format!("http://127.0.0.1:{}", port),
    };

    let url_enc = urlencoding::encode(&url);
    let token_enc = urlencoding::encode(&token);
    Ok(format!("nexus://pair?url={}&token={}", url_enc, token_enc))
}

/// Print QR code to terminal AND write a crisp SVG to /tmp for browser-scanning.
pub fn print_qr(data: &str) {
    use qrcode::QrCode;
    use qrcode::render::svg;
    use std::fs;

    let code = QrCode::new(data.as_bytes()).expect("QR-Code Generierung fehlgeschlagen");

    let svg_string = code.render::<svg::Color>()
        .min_dimensions(400, 400)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    let svg_path = "/tmp/nexus-pair.svg";
    if fs::write(svg_path, &svg_string).is_ok() {
        let _ = webbrowser::open(svg_path);
        println!("📱 QR-Code im Browser geöffnet: {svg_path}");
    }

    let colors = code.to_colors();
    let width = (colors.len() as f64).sqrt() as usize;
    let quiet = 2usize;

    let is_dark = |x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || (x as usize) >= width || (y as usize) >= width {
            return false;
        }
        colors[(y as usize) * width + (x as usize)] == qrcode::Color::Dark
    };

    let mut out = String::from("\n");
    let total = width as i32 + 2 * quiet as i32;
    let mut y = -(quiet as i32);
    while y < width as i32 + quiet as i32 {
        for x in -(quiet as i32)..(width as i32 + quiet as i32) {
            let top = is_dark(x, y);
            let bot = is_dark(x, y + 1);
            out.push(match (top, bot) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        out.push('\n');
        y += 2;
    }
    let _ = total;

    println!("{out}");
    println!("\nPairing-Daten: {data}\n");
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Axum middleware: verify Bearer token on API routes.
pub async fn require_token(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Health and dashboard are always public
    let path = req.uri().path().to_string();
    if path == "/health" || path == "/" {
        return Ok(next.run(req).await);
    }

    // Token MUST exist for protected endpoints — if missing, deny access
    let token = match fs::read_to_string(token_path()) {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => {
            tracing::error!("Kein Pairing-Token gefunden — alle API-Zugriffe blockiert");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let provided = &header[7..];
            if constant_time_eq(provided, &token) {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
