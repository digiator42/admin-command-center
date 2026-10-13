//! ACC backend API contract.
//!
//! # PENDING BACKEND (api.md not yet delivered)
//!
//! The auth endpoints below are **assumed** from the agreed direction (JSON
//! login, JWT-ready). Reconcile on arrival of `api.md`; only this file should
//! need touching:
//!
//! - `POST /api/auth/login` `{username, password}` → `200`
//!   `{token, token_type: "Bearer", expires_in, user: {id, username, role}}`,
//!   `401 {error: "invalid_credentials"}`.
//! - `POST /api/auth/refresh` (httpOnly refresh cookie, `credentials:
//!   include`) → same shape as login. Best-effort: absent endpoint just
//!   means sessions last until access-token expiry, then re-login.
//! - `GET /api/auth/me` (`Authorization: Bearer`) → `{id, username, role}`.
//! - `GET /api/services` (`Authorization: Bearer`) → `{services:
//!   [{id, name, target_url, expected_status, ping_interval_seconds,
//!   current_status, updated_at}]}`. The dashboard currently server-renders
//!   this table; the SPA needs it as JSON.
//! - Telemetry socket accepts `?token=<access-token>` (browsers cannot set
//!   headers on `new WebSocket()`), same JSON frames as today.
//! - Control `POST`s accept `Authorization: Bearer` instead of
//!   `X-CSRF-Token`. Bearer auth is not ambient (unlike cookies), so CSRF
//!   does not apply to these calls.
//!
//! Already shipped backend (verified in source):
//! - `GET /api/live-telemetry` — WS, 1/sec frames.
//! - `POST /api/admin/flush-counters` (Operator), `POST
//!   /api/admin/emergency-shutdown` (Admin) — JSON `{status, message}`.

use serde::Deserialize;

use crate::state::User;

/// Backend base URL (dev). Empty means same-origin.
pub const BASE_URL: &str = "http://127.0.0.1:8000";

pub const LOGIN_PATH: &str = "/api/auth/login";
pub const REFRESH_PATH: &str = "/api/auth/refresh";
pub const ME_PATH: &str = "/api/auth/me";
pub const SERVICES_PATH: &str = "/api/services";
pub const TELEMETRY_PATH: &str = "/api/live-telemetry";
pub const FLUSH_PATH: &str = "/api/admin/flush-counters";
pub const SHUTDOWN_PATH: &str = "/api/admin/emergency-shutdown";

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
    #[serde(default)]
    pub user: User,
}

/// JSON login. On success the caller stores the token in memory (never in
/// localStorage) and renders the dashboard.
pub async fn login(username: &str, password: &str) -> Result<LoginReply, ApiError> {
    let body = serde_json::json!({ "username": username, "password": password });
    Ok(veloasm::post_json(&with_base(LOGIN_PATH), &body).await?)
}

/// Silent refresh via the httpOnly cookie. Best-effort: any failure just
/// means "log in again".
pub async fn refresh() -> Result<LoginReply, ApiError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_credentials(web_sys::RequestCredentials::Include);
    let request = web_sys::Request::new_with_str_and_init(&with_base(REFRESH_PATH), &init)
        .map_err(|_| ApiError {
            status: 0,
            message: "bad refresh request".to_string(),
        })?;
    let window = web_sys::window().ok_or(ApiError {
        status: 0,
        message: "no window object".to_string(),
    })?;
    let response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| ApiError {
            status: 0,
            message: e.as_string().unwrap_or_else(|| "refresh failed".to_string()),
        })?;
    let response: web_sys::Response = response.dyn_into().map_err(|_| ApiError {
        status: 0,
        message: "refresh resolved to a non-Response".to_string(),
    })?;
    if !response.ok() {
        return Err(ApiError {
            status: response.status(),
            message: "session expired".to_string(),
        });
    }
    let json = JsFuture::from(response.json().map_err(|_| ApiError {
        status: 0,
        message: "bad refresh payload".to_string(),
    })?)
    .await
    .map_err(|_| ApiError {
        status: 0,
        message: "bad refresh payload".to_string(),
    })?;
    serde_json::from_str(
        &js_sys::JSON::stringify(&json)
            .map_err(|_| ApiError {
                status: 0,
                message: "bad refresh payload".to_string(),
            })?
            .as_string()
            .unwrap_or_default(),
    )
    .map_err(|e| ApiError {
        status: 0,
        message: e.to_string(),
    })
}

pub async fn me(token: &str) -> Result<User, ApiError> {
    Ok(veloasm::fetch_json(&authed(ME_PATH, token).url()).await?)
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
    let reply: ServicesReply = veloasm::fetch_json(&authed(SERVICES_PATH, token).url()).await?;
    Ok(reply.services)
}

#[derive(Debug, Deserialize)]
struct MessageReply {
    #[serde(default)]
    pub message: String,
}

/// Operator+: reset telemetry counters.
pub async fn flush_counters(token: &str) -> Result<String, ApiError> {
    let reply: MessageReply = veloasm::post_json(
        &authed(FLUSH_PATH, token).url(),
        &serde_json::json!({}),
    )
    .await?;
    Ok(reply.message)
}

/// Admin: terminate the engine kernel.
pub async fn emergency_shutdown(token: &str) -> Result<String, ApiError> {
    let reply: MessageReply = veloasm::post_json(
        &authed(SHUTDOWN_PATH, token).url(),
        &serde_json::json!({}),
    )
    .await?;
    Ok(reply.message)
}

/// Telemetry WebSocket URL with the token as query (headers impossible).
/// Derives ws(s):// from the http(s):// base.
pub fn telemetry_url(token: &str) -> String {
    let http = if token.is_empty() {
        with_base(TELEMETRY_PATH)
    } else {
        format!("{}?token={token}", with_base(TELEMETRY_PATH))
    };
    if let Some(rest) = http.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = http.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        http
    }
}
