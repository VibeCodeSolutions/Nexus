use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::fs;
use std::path::PathBuf;

fn token_path() -> PathBuf {
    let dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    dir.join(".nexus_token")
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// Generate a random pairing token and store it.
pub fn generate_token() -> Result<String, String> {
    use base64::Engine;
    use rand::Rng;

    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    fs::write(token_path(), &token)
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

/// Get pairing info as JSON string for QR code.
pub fn pairing_json(bind_addr: &str) -> Result<String, String> {
    let token = get_or_create_token()?;

    let port = bind_addr.split(':').last().unwrap_or("7777");
    let url = match local_ip_address::local_ip() {
        Ok(ip) => format!("http://{}:{}", ip, port),
        Err(_) => format!("http://127.0.0.1:{}", port),
    };

    Ok(format!(r#"{{"url":"{}","token":"{}"}}"#, url, token))
}

/// Print QR code to terminal.
pub fn print_qr(data: &str) {
    use qrcode::QrCode;

    let code = QrCode::new(data.as_bytes()).expect("QR-Code Generierung fehlgeschlagen");
    let string = code
        .render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .build();

    println!("\n{string}");
    println!("\nPairing-Daten: {data}\n");
}

/// Axum middleware: verify Bearer token on API routes.
pub async fn require_token(req: Request, next: Next) -> Result<Response, StatusCode> {
    // Health and dashboard are always public
    let path = req.uri().path().to_string();
    if path == "/health" || path == "/" {
        return Ok(next.run(req).await);
    }

    let token = match fs::read_to_string(token_path()) {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => return Ok(next.run(req).await), // No token file = no auth required
    };

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let provided = &header[7..];
            if provided == token {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
