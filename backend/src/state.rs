use std::sync::Arc;

use chrono::Utc;
use gritshield::core::ioc::CONTEXT;

/// A downstream Grit platform product that ACC can probe and proxy to.
/// Endpoints and credentials are supplied purely through the environment so the
/// same image works in the GritSoc compose network or on bare metal.
#[derive(Clone)]
pub struct Connector {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
}

impl Connector {
    /// Join the connector base URL with an upstream path, tolerating a trailing
    /// slash on the base and a missing leading slash on the path.
    pub fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

/// Application-wide shared state, registered once into the GritShield DI
/// container and resolved on demand by handlers and middlewares.
#[derive(Clone)]
pub struct AppState {
    /// Shared HTTP client used to probe and proxy the downstream products.
    pub http: reqwest::Client,
    /// HMAC secret used to sign and verify the stateless JWT access tokens.
    pub jwt_secret: String,
    /// Downstream connectors, keyed by short name (`luxshield`, `cyberflow`, ...).
    pub connectors: Arc<Vec<Connector>>,
    /// Unix timestamp (seconds) captured at boot for uptime reporting.
    pub started_at: i64,
}

impl AppState {
    pub fn new(jwt_secret: String, connectors: Vec<Connector>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .expect("failed to build the shared reqwest client");

        Self {
            http,
            jwt_secret,
            connectors: Arc::new(connectors),
            started_at: Utc::now().timestamp(),
        }
    }

    pub fn connector(&self, name: &str) -> Option<&Connector> {
        self.connectors
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn uptime_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.started_at
    }
}

/// Resolve the registered application state from the global container.
pub fn resolve() -> Arc<AppState> {
    CONTEXT
        .resolve::<AppState>()
        .expect("AppState was not registered; this must be the first boot step")
}
