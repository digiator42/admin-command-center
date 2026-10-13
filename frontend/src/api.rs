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
pub const LOGOUT_PATH: &str = "/api/v1/auth/logout";
/// Boot-time token validation (wired when the app needs it).
#[allow(dead_code)]
pub const ME_PATH: &str = "/api/v1/auth/me";
pub const SERVICES_PATH: &str = "/api/v1/services";
pub const TELEMETRY_PATH: &str = "/api/live-telemetry";
pub const FLUSH_PATH: &str = "/api/v1/control/flush-counters";
pub const SHUTDOWN_PATH: &str = "/api/v1/control/emergency-shutdown";
pub const BLOCK_IP_PATH: &str = "/api/v1/control/block-ip";
pub const UNBLOCK_IP_PATH: &str = "/api/v1/control/unblock-ip";
pub const USERS_PATH: &str = "/api/v1/users";
pub const DOCS_PATH: &str = "/api/v1/docs";
pub const SYSTEM_METRICS_PATH: &str = "/api/v1/system/metrics";
pub const SYSTEM_INFO_PATH: &str = "/api/v1/system/info";
pub const INTEGRATIONS_PATH: &str = "/api/v1/integrations";

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

fn authed(path: impl AsRef<str>, token: &str) -> veloasm::RequestBuilder {
    veloasm::request(&with_base(path.as_ref())).header("Authorization", &format!("Bearer {token}"))
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

/// Stateless logout: the server has nothing to revoke; still called so the
/// audit trail sees the sign-out. Result ignored — local logout proceeds
/// regardless.
pub async fn logout(token: &str) {
    let Ok(req) = authed(LOGOUT_PATH, token)
        .method("POST")
        .json_body(&serde_json::json!({}))
    else {
        return;
    };
    let _: Result<serde_json::Value, _> = req.send_json().await;
}

// ---------------------------------------------------------------------------
// Services + heartbeats
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Heartbeat {
    pub id: i64,
    pub service_id: i32,
    pub latency_ms: i64,
    pub http_status_code: i16,
    pub status_msg: String,
    pub error_log: Option<String>,
    pub checked_at: String,
}

#[derive(Debug, Deserialize)]
struct HeartbeatsReply {
    #[serde(default)]
    pub heartbeats: Vec<Heartbeat>,
}

/// Operator+: register a monitored service.
pub async fn create_service(
    token: &str,
    name: &str,
    target_url: &str,
) -> Result<crate::state::MonitoredService, ApiError> {
    #[derive(Deserialize)]
    struct CreateReply {
        service: crate::state::MonitoredService,
    }
    let body = serde_json::json!({ "name": name, "target_url": target_url });
    let reply: CreateReply = authed(SERVICES_PATH, token)
        .method("POST")
        .json_body(&body)?
        .send_json()
        .await?;
    Ok(reply.service)
}

/// Admin: remove a service (cascades to its heartbeats).
pub async fn delete_service(token: &str, id: i32) -> Result<(), ApiError> {
    authed(format!("{}/{id}", with_base(SERVICES_PATH)), token)
        .method("DELETE")
        .send()
        .await?;
    Ok(())
}

/// Newest-first heartbeat history for one service.
pub async fn service_heartbeats(
    token: &str,
    id: i32,
    limit: u32,
) -> Result<Vec<Heartbeat>, ApiError> {
    let reply: HeartbeatsReply = authed(
        format!(
            "{}/{id}/heartbeats?limit={limit}",
            with_base(SERVICES_PATH)
        ),
        token,
    )
    .send_json()
    .await?;
    Ok(reply.heartbeats)
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct SystemMetrics {
    pub active_connections: u64,
    pub total_requests: u64,
    pub total_blocked_ips: u64,
    pub total_rate_limited_requests: u64,
    pub total_errors_4xx: u64,
    pub total_errors_5xx: u64,
    pub avg_latency_ms: f64,
    pub uptime_seconds: u64,
    pub services_total: u64,
    pub services_up: u64,
}

pub async fn system_metrics(token: &str) -> Result<SystemMetrics, ApiError> {
    Ok(authed(SYSTEM_METRICS_PATH, token).send_json().await?)
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct SystemInfo {
    pub version: String,
    pub uptime_seconds: u64,
}

pub async fn system_info(token: &str) -> Result<SystemInfo, ApiError> {
    Ok(authed(SYSTEM_INFO_PATH, token).send_json().await?)
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ConnectorInfo {
    pub name: String,
    pub url: String,
    pub reachable: bool,
    pub status_code: u16,
    pub latency_ms: u64,
}

#[derive(Debug, Deserialize)]
struct IntegrationsReply {
    #[serde(default)]
    pub integrations: Vec<ConnectorInfo>,
}

/// Downstream connector health (luxshield, cyberflow, gritirt, gritjira).
pub async fn integrations(token: &str) -> Result<Vec<ConnectorInfo>, ApiError> {
    let reply: IntegrationsReply = authed(INTEGRATIONS_PATH, token).send_json().await?;
    Ok(reply.integrations)
}

// ---------------------------------------------------------------------------
// Users (Admin)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ManagedUser {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct UsersReply {
    #[serde(default)]
    pub users: Vec<ManagedUser>,
}

pub async fn list_users(token: &str) -> Result<Vec<ManagedUser>, ApiError> {
    let reply: UsersReply = authed(USERS_PATH, token).send_json().await?;
    Ok(reply.users)
}

pub async fn create_user(
    token: &str,
    username: &str,
    password: &str,
    role: &str,
) -> Result<ManagedUser, ApiError> {
    #[derive(Deserialize)]
    struct CreateReply {
        user: ManagedUser,
    }
    let body = serde_json::json!({ "username": username, "password": password, "role": role });
    let reply: CreateReply = authed(USERS_PATH, token)
        .method("POST")
        .json_body(&body)?
        .send_json()
        .await?;
    Ok(reply.user)
}

pub async fn delete_user(token: &str, id: i64) -> Result<(), ApiError> {
    authed(format!("{}/{id}", with_base(USERS_PATH)), token)
        .method("DELETE")
        .send()
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Platform docs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DocMeta {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Doc {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub markdown_body: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct DocsReply {
    #[serde(default)]
    pub docs: Vec<DocMeta>,
}

pub async fn list_docs(token: &str) -> Result<Vec<DocMeta>, ApiError> {
    let reply: DocsReply = authed(DOCS_PATH, token).send_json().await?;
    Ok(reply.docs)
}

pub async fn get_doc(token: &str, slug: &str) -> Result<Doc, ApiError> {
    #[derive(Deserialize)]
    struct DocReply {
        doc: Doc,
    }
    let reply: DocReply =
        authed(format!("{}?slug={slug}", with_base(DOCS_PATH)), token)
            .send_json()
            .await?;
    Ok(reply.doc)
}

pub async fn create_doc(
    token: &str,
    slug: &str,
    title: &str,
    markdown_body: &str,
) -> Result<Doc, ApiError> {
    #[derive(Deserialize)]
    struct CreateReply {
        doc: Doc,
    }
    let body = serde_json::json!({ "slug": slug, "title": title, "markdown_body": markdown_body });
    let reply: CreateReply = authed(DOCS_PATH, token)
        .method("POST")
        .json_body(&body)?
        .send_json()
        .await?;
    Ok(reply.doc)
}

pub async fn delete_doc(token: &str, slug: &str) -> Result<(), ApiError> {
    authed(format!("{}?slug={slug}", with_base(DOCS_PATH)), token)
        .method("DELETE")
        .send()
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Firewall proxy (Operator+)
// ---------------------------------------------------------------------------

/// Proxy a block rule to LuxShield.
pub async fn block_ip(token: &str, ip: &str, reason: &str) -> Result<(), ApiError> {
    let body = serde_json::json!({ "ip": ip, "reason": reason });
    let _: serde_json::Value = authed(BLOCK_IP_PATH, token)
        .method("POST")
        .json_body(&body)?
        .send_json()
        .await?;
    Ok(())
}

/// Proxy an unblock to LuxShield.
pub async fn unblock_ip(token: &str, ip: &str) -> Result<(), ApiError> {
    authed(format!("{UNBLOCK_IP_PATH}?ip={ip}"), token)
        .method("POST")
        .send()
        .await?;
    Ok(())
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
