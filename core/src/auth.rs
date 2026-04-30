use axum::extract::{ConnectInfo, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::home_dir;

pub fn token_path() -> PathBuf {
    home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus_token")
}

pub fn paired_path() -> PathBuf {
    home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nexus_paired_at")
}

/// Write the current Unix timestamp to ~/.nexus_paired_at to record that
/// a non-localhost client successfully authenticated. Errors are logged
/// and swallowed — failure to update this file must never break auth.
pub fn mark_paired_now() {
    let ts = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return,
    };
    let path = paired_path();
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    match opts
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, ts.to_string().as_bytes()))
    {
        Ok(()) => tracing::info!("Pairing markiert: {} ({})", path.display(), ts),
        Err(e) => tracing::warn!("paired_at konnte nicht geschrieben werden: {e}"),
    }
}

/// Read the last-paired timestamp, if any.
pub fn paired_at() -> Option<u64> {
    fs::read_to_string(paired_path())
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
}

/// Generate a random pairing token and store it with restrictive permissions.
pub fn generate_token() -> Result<String, String> {
    use base64::Engine;
    use rand::Rng;

    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let path = token_path();
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    opts.open(&path)
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

    let port = bind_addr.split(':').next_back().unwrap_or("7777");
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

fn is_loopback(addr: &SocketAddr) -> bool {
    addr.ip().is_loopback()
}

/// Axum middleware: verify Bearer token on API routes.
///
/// Public paths (`/health`, `/api/setup-status`) are always allowed —
/// these are unauthenticated probes the Wizard and any LAN peer may hit.
/// The dashboard root `/` is *not* public: it renders all stored
/// braindumps, projects and stats, and the default bind is `0.0.0.0`,
/// so leaving `/` unauthenticated would leak everything to anyone on the
/// same network. Bearer auth required.
///
/// If a request to *any* path carries a valid Bearer from a non-loopback
/// peer, we record it as a pairing event. The Android client makes this
/// explicit via `POST /api/pair/handshake` right after consuming the QR
/// — that handshake is what the Wizard's `paired`-poll waits for.
pub async fn require_token(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let is_public = path == "/health" || path == "/api/setup-status";

    let stored = fs::read_to_string(token_path())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let has_auth = auth_header.is_some();
    let bearer_valid = match (&stored, auth_header) {
        (Some(token), Some(h)) if h.starts_with("Bearer ") => {
            constant_time_eq(&h[7..], token)
        }
        _ => false,
    };

    let loopback = is_loopback(&addr);
    tracing::debug!(
        "auth: path={} peer={} loopback={} has_auth={} bearer_valid={}",
        path, addr, loopback, has_auth, bearer_valid
    );
    if has_auth && !bearer_valid {
        tracing::warn!("auth: ungültiger Bearer von peer={} path={}", addr, path);
    }

    // Track pairing whenever a remote client presents a valid token,
    // regardless of which endpoint they hit.
    if bearer_valid && !loopback {
        tracing::info!("auth: pairing-event von peer={} path={}", addr, path);
        mark_paired_now();
    }

    if is_public {
        return Ok(next.run(req).await);
    }

    // Protected endpoints require a stored token AND a valid Bearer.
    if stored.is_none() {
        tracing::error!("Kein Pairing-Token gefunden — alle API-Zugriffe blockiert");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if bearer_valid {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
