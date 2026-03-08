use axum::{
    body::Body,
    extract::{ConnectInfo, FromRequest, State},
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    Form,
};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::storage::{SecretsStore, SessionStore};

const SESSION_COOKIE_NAME: &str = "twolebot_session";
const SESSION_MAX_AGE_SECS: u64 = 90 * 24 * 60 * 60; // 90 days

/// Rate limiter: max attempts per IP within a sliding window.
const RATE_LIMIT_MAX_ATTEMPTS: usize = 10;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(300); // 5 minutes

/// Nonce TTL: how long a QR login nonce is valid.
const NONCE_TTL: Duration = Duration::from_secs(600); // 10 minutes

/// Simple in-memory rate limiter for auth endpoints.
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP → list of attempt timestamps (within window)
    attempts: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record an attempt and return whether the request is allowed.
    pub async fn check_and_record(&self, ip: IpAddr) -> bool {
        let mut map = self.attempts.lock().await;
        let now = Instant::now();
        let cutoff = now - RATE_LIMIT_WINDOW;

        let attempts = map.entry(ip).or_default();
        // Remove expired attempts
        attempts.retain(|t| *t > cutoff);

        if attempts.len() >= RATE_LIMIT_MAX_ATTEMPTS {
            return false; // Rate limited
        }

        attempts.push(now);
        true
    }
}

/// One-time nonce store for QR login URLs.
/// Nonces are consumed on use and expire after NONCE_TTL.
#[derive(Clone)]
pub struct NonceStore {
    nonces: Arc<Mutex<HashMap<String, Instant>>>,
}

impl NonceStore {
    pub fn new() -> Self {
        Self {
            nonces: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate a new nonce and return it.
    pub async fn generate(&self) -> String {
        let nonce = uuid::Uuid::new_v4().to_string();
        let mut map = self.nonces.lock().await;

        // Cleanup expired nonces while we're here
        let cutoff = Instant::now() - NONCE_TTL;
        map.retain(|_, created| *created > cutoff);

        map.insert(nonce.clone(), Instant::now());
        nonce
    }

    /// Validate a nonce: returns true if valid (and consumes it).
    pub async fn validate_and_consume(&self, nonce: &str) -> bool {
        let mut map = self.nonces.lock().await;
        if let Some(created) = map.remove(nonce) {
            // Check it hasn't expired
            created.elapsed() < NONCE_TTL
        } else {
            false
        }
    }
}

/// Shared auth state passed to middleware and handlers.
#[derive(Clone)]
pub struct AuthState {
    pub secrets: Arc<SecretsStore>,
    pub sessions: Arc<SessionStore>,
    pub rate_limiter: RateLimiter,
    pub nonce_store: NonceStore,
}

/// Auth middleware: skip auth for localhost, require valid session for external.
///
/// Only trusts X-Forwarded-For when the direct TCP peer is loopback (i.e., behind a
/// reverse proxy like cloudflared). If the peer is non-loopback, the request is
/// external regardless of any forwarded headers (prevents header spoofing).
pub async fn auth_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(auth): State<AuthState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let is_local = if addr.ip().is_loopback() {
        // Direct peer is localhost — we might be behind a reverse proxy.
        // Only now trust X-Forwarded-For to determine the real client.
        if let Some(forwarded) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            forwarded
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .all(|ip| ip.parse::<IpAddr>().map(|a| a.is_loopback()).unwrap_or(false))
        } else {
            // Peer is localhost, no forwarded header — genuinely local
            true
        }
    } else {
        // Direct peer is non-loopback — external, regardless of headers
        false
    };

    if is_local {
        return next.run(req).await;
    }

    // External request — check session cookie
    let session_valid = extract_session_cookie(&req)
        .and_then(|sid| auth.sessions.validate(&sid).ok())
        .unwrap_or(false);

    if session_valid {
        return next.run(req).await;
    }

    // Not authenticated — return 401
    (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
}

/// Extract the real client IP for rate limiting purposes.
fn extract_client_ip(addr: &SocketAddr, req: &Request<Body>) -> IpAddr {
    if addr.ip().is_loopback() {
        // Behind proxy — use first IP from X-Forwarded-For if available
        if let Some(forwarded) = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(first_ip) = forwarded.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }
    addr.ip()
}

/// Extract the session ID from the Cookie header.
fn extract_session_cookie(req: &Request<Body>) -> Option<String> {
    let cookie_header = req.headers().get(header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(SESSION_COOKIE_NAME) {
            let value = value.strip_prefix('=')?;
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// GET /auth/login — serves the login page HTML.
pub async fn login_page() -> Html<String> {
    Html(LOGIN_HTML.to_string())
}

/// POST /auth/login — validates token, creates session, sets cookie.
#[derive(serde::Deserialize)]
pub struct LoginForm {
    token: String,
}

pub async fn login_submit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(auth): State<AuthState>,
    req: Request<Body>,
) -> Response {
    // Rate limit check
    let client_ip = extract_client_ip(&addr, &req);
    if !auth.rate_limiter.check_and_record(client_ip).await {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts. Try again later.").into_response();
    }

    let is_https = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("https"))
        .unwrap_or(false);

    let form: LoginForm = match Form::from_request(req, &()).await {
        Ok(Form(f)) => f,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid form data").into_response(),
    };
    let stored_token = match auth.secrets.get_auth_token() {
        Ok(Some(t)) => t,
        _ => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Auth not configured").into_response();
        }
    };

    if form.token != stored_token {
        return Html(LOGIN_HTML_WRONG_TOKEN.to_string()).into_response();
    }

    // Valid token — create session
    let session_id = match auth.sessions.create() {
        Ok(sid) => sid,
        Err(e) => {
            tracing::error!("Failed to create session: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response();
        }
    };

    // Clean up old sessions (non-fatal)
    if let Err(e) = auth.sessions.cleanup_expired() {
        tracing::warn!("Failed to cleanup expired sessions: {}", e);
    }

    // Set cookie and redirect to dashboard
    let secure_flag = if is_https { "; Secure" } else { "" };
    let cookie = format!(
        "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        SESSION_COOKIE_NAME, session_id, SESSION_MAX_AGE_SECS, secure_flag,
    );

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "redirect failed").into_response())
}

/// GET /auth/token?token=X — validates auth token directly, creates session, redirects to /chat.
/// Used for shareable login links (e.g., giving someone a direct access URL).
#[derive(serde::Deserialize)]
pub struct TokenLoginQuery {
    token: String,
}

pub async fn token_login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(auth): State<AuthState>,
    query: axum::extract::Query<TokenLoginQuery>,
    req: Request<Body>,
) -> Response {
    let client_ip = extract_client_ip(&addr, &req);
    if !auth.rate_limiter.check_and_record(client_ip).await {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts. Try again later.").into_response();
    }

    // Validate token against stored auth token
    let stored_token = match auth.secrets.get_auth_token() {
        Ok(Some(t)) => t,
        _ => return (StatusCode::UNAUTHORIZED, "Auth not configured").into_response(),
    };

    if query.token != stored_token {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }

    let session_id = match auth.sessions.create() {
        Ok(sid) => sid,
        Err(e) => {
            tracing::error!("Token login: failed to create session: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response();
        }
    };

    if let Err(e) = auth.sessions.cleanup_expired() {
        tracing::warn!("Token login: failed to cleanup expired sessions: {}", e);
    }

    let is_https = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("https"))
        .unwrap_or(false);
    let secure_flag = if is_https { "; Secure" } else { "" };

    let cookie = format!(
        "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        SESSION_COOKIE_NAME, session_id, SESSION_MAX_AGE_SECS, secure_flag,
    );

    tracing::info!("Token login successful, redirecting to /chat");

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/chat")
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "redirect failed").into_response())
}

/// GET /auth/qr?nonce=X — validates a one-time nonce, creates session, redirects to /chat.
/// Used by QR code scanning: phone scans QR → opens this URL → gets session cookie → lands on chat.
#[derive(serde::Deserialize)]
pub struct QrLoginQuery {
    nonce: String,
}

pub async fn qr_login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(auth): State<AuthState>,
    query: axum::extract::Query<QrLoginQuery>,
    req: Request<Body>,
) -> Response {
    // Rate limit check
    let client_ip = extract_client_ip(&addr, &req);
    if !auth.rate_limiter.check_and_record(client_ip).await {
        return (StatusCode::TOO_MANY_REQUESTS, "Too many login attempts. Try again later.").into_response();
    }

    // Validate one-time nonce (consumes it on success)
    if !auth.nonce_store.validate_and_consume(&query.nonce).await {
        return (StatusCode::UNAUTHORIZED, "Invalid or expired login link").into_response();
    }

    // Valid nonce — create session
    let session_id = match auth.sessions.create() {
        Ok(sid) => sid,
        Err(e) => {
            tracing::error!("QR login: failed to create session: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed").into_response();
        }
    };

    if let Err(e) = auth.sessions.cleanup_expired() {
        tracing::warn!("QR login: failed to cleanup expired sessions: {}", e);
    }

    // Tunnel traffic comes through Cloudflare, which sets X-Forwarded-Proto
    let is_https = req
        .headers()
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("https"))
        .unwrap_or(false);
    let secure_flag = if is_https { "; Secure" } else { "" };

    let cookie = format!(
        "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        SESSION_COOKIE_NAME, session_id, SESSION_MAX_AGE_SECS, secure_flag,
    );

    tracing::info!("QR login successful, redirecting to /chat");

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/chat")
        .header(header::SET_COOKIE, cookie)
        .body(Body::empty())
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "redirect failed").into_response())
}

const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>twolebot — Login</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f1117;
    color: #e1e4e8;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
  }
  .card {
    background: #1c1f26;
    border: 1px solid #2d3139;
    border-radius: 12px;
    padding: 2rem;
    width: 100%;
    max-width: 400px;
    margin: 1rem;
  }
  h1 { font-size: 1.3rem; margin-bottom: 0.5rem; }
  .hint {
    color: #8b949e;
    font-size: 0.85rem;
    margin-bottom: 1.5rem;
    line-height: 1.5;
  }
  .hint code {
    background: #2d3139;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.8rem;
  }
  label { display: block; font-size: 0.9rem; margin-bottom: 0.4rem; }
  input[type="password"] {
    width: 100%;
    padding: 0.6rem 0.8rem;
    background: #0f1117;
    border: 1px solid #2d3139;
    border-radius: 6px;
    color: #e1e4e8;
    font-size: 1rem;
    margin-bottom: 1rem;
  }
  input[type="password"]:focus {
    outline: none;
    border-color: #58a6ff;
  }
  button {
    width: 100%;
    padding: 0.6rem;
    background: #238636;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
  }
  button:hover { background: #2ea043; }
  .error {
    background: #3d1f1f;
    border: 1px solid #6e3030;
    color: #f88;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
</style>
</head>
<body>
<div class="card">
  <h1>twolebot</h1>
  <p class="hint">
    Enter the auth token shown at server startup,<br>
    or run <code>twolebot status</code> on the server.
  </p>
  <form method="POST" action="/auth/login">
    <label for="token">Auth token</label>
    <input type="password" id="token" name="token"
           autocomplete="current-password" required autofocus>
    <button type="submit">Log in</button>
  </form>
</div>
</body>
</html>"#;

const LOGIN_HTML_WRONG_TOKEN: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>twolebot — Login</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f1117;
    color: #e1e4e8;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
  }
  .card {
    background: #1c1f26;
    border: 1px solid #2d3139;
    border-radius: 12px;
    padding: 2rem;
    width: 100%;
    max-width: 400px;
    margin: 1rem;
  }
  h1 { font-size: 1.3rem; margin-bottom: 0.5rem; }
  .hint {
    color: #8b949e;
    font-size: 0.85rem;
    margin-bottom: 1.5rem;
    line-height: 1.5;
  }
  .hint code {
    background: #2d3139;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.8rem;
  }
  label { display: block; font-size: 0.9rem; margin-bottom: 0.4rem; }
  input[type="password"] {
    width: 100%;
    padding: 0.6rem 0.8rem;
    background: #0f1117;
    border: 1px solid #2d3139;
    border-radius: 6px;
    color: #e1e4e8;
    font-size: 1rem;
    margin-bottom: 1rem;
  }
  input[type="password"]:focus {
    outline: none;
    border-color: #58a6ff;
  }
  button {
    width: 100%;
    padding: 0.6rem;
    background: #238636;
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 1rem;
    cursor: pointer;
  }
  button:hover { background: #2ea043; }
  .error {
    background: #3d1f1f;
    border: 1px solid #6e3030;
    color: #f88;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 0.85rem;
    margin-bottom: 1rem;
  }
</style>
</head>
<body>
<div class="card">
  <h1>twolebot</h1>
  <p class="hint">
    Enter the auth token shown at server startup,<br>
    or run <code>twolebot status</code> on the server.
  </p>
  <div class="error">Invalid token. Please try again.</div>
  <form method="POST" action="/auth/login">
    <label for="token">Auth token</label>
    <input type="password" id="token" name="token"
           autocomplete="current-password" required autofocus>
    <button type="submit">Log in</button>
  </form>
</div>
</body>
</html>"#;
