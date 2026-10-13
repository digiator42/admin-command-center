//! Reactive application state for the ACC console.
//!
//! One [`AppState`] value holds every signal, provided once via `provide!`
//! and pulled out of context with `use_context::<AppState>()`.
//!
//! Auth is token-in-memory (never localStorage): [`Auth`] lives in a signal,
//! REST calls attach `Authorization: Bearer`, and the telemetry socket takes
//! `?token=`. A 401 anywhere drops back to the login view.

use std::cell::Cell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use veloasm::prelude::*;

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
}

impl User {
    pub fn is_operator(&self) -> bool {
        self.role == "Operator" || self.role == "Admin"
    }

    pub fn is_admin(&self) -> bool {
        self.role == "Admin"
    }
}

/// Live session. The token stays in memory only — it is never persisted to
/// localStorage/sessionStorage (XSS can read those; memory cannot outlive
/// the page).
#[derive(Clone, Debug)]
pub struct Auth {
    pub token: String,
    pub user: User,
    /// Epoch millis when the token expires (0 = unknown). Kept for contract
    /// fidelity with the login reply; no pro-active refresh yet.
    #[allow(dead_code)]
    pub expires_at_ms: i64,
}

// ---------------------------------------------------------------------------
// Telemetry + inventory
// ---------------------------------------------------------------------------

/// One telemetry frame from `ws(s)://host/api/live-telemetry`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Telemetry {
    pub active_connections: u64,
    pub total_blocked_ips: u64,
    pub cpu_worker_latency_ms: u64,
    pub memory_usage_bytes: u64,
    pub total_rate_limited_reqs: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoredService {
    pub id: i32,
    pub name: String,
    pub target_url: String,
    pub expected_status: i16,
    pub ping_interval_seconds: i32,
    pub current_status: String,
    pub updated_at: Option<String>,
}

impl MonitoredService {
    /// Full Tailwind badge literals (kept whole for the Tailwind scanner).
    pub fn badge(&self) -> &'static str {
        match self.current_status.as_str() {
            "UP" => "bg-green-500/10 text-green-400",
            "DEGRADED" => "bg-yellow-500/10 text-yellow-400",
            _ => "bg-red-500/10 text-red-400",
        }
    }

    pub fn dot(&self) -> &'static str {
        match self.current_status.as_str() {
            "UP" => "bg-green-400 animate-pulse",
            "DEGRADED" => "bg-yellow-400 animate-pulse",
            _ => "bg-red-400",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub detail: String,
    pub bad: bool,
}

pub const TOAST_KEEP: usize = 5;
pub const TOAST_TTL_MS: u32 = 6_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnStatus {
    Connecting,
    Open,
    Reconnecting,
    Closed,
}

impl ConnStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConnStatus::Connecting => "CONNECTING",
            ConnStatus::Open => "LIVE",
            ConnStatus::Reconnecting => "RECONNECTING",
            ConnStatus::Closed => "OFFLINE",
        }
    }

    pub fn dot(&self) -> &'static str {
        match self {
            ConnStatus::Connecting => "bg-amber-400 dot-busy",
            ConnStatus::Open => "bg-emerald-400",
            ConnStatus::Reconnecting => "bg-amber-400 dot-busy",
            ConnStatus::Closed => "bg-rose-500",
        }
    }
}

// ---------------------------------------------------------------------------
// Signals
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    /// `None` = show the login view.
    pub auth: RwSignal<Option<Auth>>,
    pub auth_error: RwSignal<String>,

    pub telemetry: RwSignal<Telemetry>,
    pub services: SignalVec<MonitoredService>,
    pub toasts: SignalVec<Toast>,

    pub conn: RwSignal<ConnStatus>,
    pub api_error: RwSignal<String>,

    pub metrics: Metrics,
}

#[derive(Clone)]
pub struct Metrics {
    pub next_id: Rc<Cell<u64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth: signal!(None),
            auth_error: signal!(String::new()),
            telemetry: signal!(Telemetry::default()),
            services: signal_vec(Vec::new()),
            toasts: signal_vec(Vec::new()),
            conn: signal!(ConnStatus::Connecting),
            api_error: signal!(String::new()),
            metrics: Metrics {
                next_id: Rc::new(Cell::new(1)),
            },
        }
    }

    pub fn next_toast_id(&self) -> u64 {
        let id = self.metrics.next_id.get();
        self.metrics.next_id.set(id + 1);
        id
    }

    #[allow(dead_code)]
    pub fn bearer(&self) -> Option<String> {
        self.auth.get().map(|a| format!("Bearer {}", a.token))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Bytes → human label (`42.0 MB`).
pub fn human_bytes(b: u64) -> String {
    const KB: f64 = 1024.0;
    let b = b as f64;
    if b >= KB * KB * KB {
        format!("{:.1} GB", b / (KB * KB * KB))
    } else if b >= KB * KB {
        format!("{:.1} MB", b / (KB * KB))
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{b:.0} B")
    }
}
