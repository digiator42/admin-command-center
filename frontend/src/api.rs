//! ACC backend API contract (see `api.md`).
//!
//! - `POST /api/v1/auth/login` `{username, password}` → `200`
//!   `{token, token_type, expires_in, expires_at, user}`, `401
//!   {error: "invalid_credentials"}`. JWT HS256, 12 h. No refresh endpoint:
//!   on 401, log in again.
//! - `GET /api/v1/auth/me` → `{user, claims}`.
//! - `GET /api/v1/services` → `{count, services: [...]}`.
//! - Telemetry WS `/api/live-telemetry` is **public** (no token needed).
//! - Control `POST /api/v1/control/flush-counters` (Operator+) and
//!   `/api/v1/control/emergency-shutdown` (Admin) take `Authorization:
//!   Bearer`. Bearer auth is not ambient (unlike cookies), so CSRF does not
//!   apply to these calls.

use serde::Deserialize;

use crate::state::User;

/// Backend base URL (dev). Empty means same-origin.
pub const BASE_URL: &str = "http://127.0.0.1:8000";

pub const LOGIN_PATH: &str = "/api/v1/auth/login";
/// Boot-time token validation (wired when the app needs it).
#[allow(dead_code)]
pub const ME_PATH: &str = "/api/v1/auth/me";
pub const SERVICES_PATH: &str = "/api/v1/services";
pub const TELEMETRY_PATH: &str = "/api/live-telemetry";
pub const FLUSH_PATH: &str = "/api/v1/control/flush-counters";
pub const SHUTDOWN_PATH: &str = "/api/v1/control/emergency-shutdown";

fn with_base(path: &str) -> String {
    if BASE_URL.is_empty() {
        path.to_string()
    } else {
        format!("{}{}", BASE_URL.trim_end_matches('/'), path)
    }
}

/// Fetch failure with the HTTP status preserved (401 = bad credentials or an
/// expired/revoked token → drop back to login).
#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub message: String,
}

impl ApiError {
    pub fn is_unauthorized(&self) -> bool {
        self.status == 401 || self.status == 403
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<veloasm::FetchError> for ApiError {
    fn from(e: veloasm::FetchError) -> Self {
        match e {
            veloasm::FetchError::Status { code, reason } => ApiError {
                status: code,
                message: if code == 401 {
                    "unauthorized: bad credentials or session expired".to_string()
                } else if code == 403 {
                    "forbidden: your role cannot perform this action".to_string()
                } else {
                    format!("request failed ({code} {reason})")
                },
            },
            veloasm::FetchError::Network(msg) => ApiError { status: 0, message: msg },
            veloasm::FetchError::Decode(msg) => ApiError { status: 0, message: msg },
        }
    }
}

fn authed(path: &str, token: &str) -> veloasm::RequestBuilder {
    veloasm::request(&with_base(path)).header("Authorization", &format!("Bearer {token}"))
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LoginReply {
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub expires_in: i64,
    /// Unix seconds (preferred over `expires_in` for expiry math).
    #[serde(default)]
    pub expires_at: i64,
    #[serde(default)]
    pub user: User,
}

/// JSON login. On success the caller stores the token in memory (never in
/// localStorage) and renders the dashboard.
pub async fn login(username: &str, password: &str) -> Result<LoginReply, ApiError> {
    let body = serde_json::json!({ "username": username, "password": password });
    Ok(veloasm::post_json(&with_base(LOGIN_PATH), &body).await?)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MeReply {
    user: User,
}

#[allow(dead_code)]
pub async fn me(token: &str) -> Result<User, ApiError> {
    let reply: MeReply = authed(ME_PATH, token).send_json().await?;
    Ok(reply.user)
}

// ---------------------------------------------------------------------------
// Data + control
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ServicesReply {
    #[serde(default)]
    pub services: Vec<crate::state::MonitoredService>,
}

pub async fn list_services(token: &str) -> Result<Vec<crate::state::MonitoredService>, ApiError> {
    let reply: ServicesReply = authed(SERVICES_PATH, token).send_json().await?;
    Ok(reply.services)
}

#[derive(Debug, Deserialize)]
struct MessageReply {
    #[serde(default)]
    pub message: String,
}

/// Operator+: reset telemetry counters.
pub async fn flush_counters(token: &str) -> Result<String, ApiError> {
    let reply: MessageReply = authed(FLUSH_PATH, token)
        .method("POST")
        .json_body(&serde_json::json!({}))?
        .send_json()
        .await?;
    Ok(reply.message)
}

/// Admin: terminate the engine kernel.
pub async fn emergency_shutdown(token: &str) -> Result<String, ApiError> {
    let reply: MessageReply = authed(SHUTDOWN_PATH, token)
        .method("POST")
        .json_body(&serde_json::json!({}))?
        .send_json()
        .await?;
    Ok(reply.message)
}

/// Telemetry WebSocket URL. The socket is public (no token needed), so this
/// is just a ws(s):// derivation of the http(s):// base.
pub fn telemetry_url() -> String {
    let http = with_base(TELEMETRY_PATH);
    if let Some(rest) = http.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = http.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        http
    }
}
