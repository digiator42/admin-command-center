mod api;
mod bootstrap;
mod middleware;
mod models;
mod security;
mod state;
mod telemetry_ws;
mod workers;

use std::sync::Arc;

use gritshield::core::logger::LogLevel;
use gritshield::middleware::AuthMiddleware;
use gritshield::prelude::*;
use gritshield::security::jwt::JwtHandler;

use crate::middleware::SecurityHeaders;
use crate::state::{AppState, Connector};

/// Build a downstream connector descriptor from the environment, falling back
/// to the compose-network default and leaving the API key optional.
fn connector(name: &str, url_env: &str, url_default: &str, key_env: &str) -> Connector {
    let base_url = std::env::var(url_env).unwrap_or_else(|_| url_default.to_string());
    let api_key = std::env::var(key_env).ok().filter(|k| !k.is_empty());
    Connector {
        name: name.to_string(),
        base_url,
        api_key,
    }
}

#[tokio::main]
async fn main() {
    let shared_db = bootstrap::connect_and_migrate_db().await;

    let monitor_db = shared_db.clone();
    tokio::spawn(async move {
        println!("[MONITOR] Launching upstream engine health worker...");
        workers::monitor::start_service_monitor_loop(monitor_db).await;
    });

    let connectors = vec![
        connector(
            "luxshield",
            "ACC_LUXSHIELD_URL",
            "http://luxshield-backend:8020",
            "ACC_LUXSHIELD_API_KEY",
        ),
        connector(
            "cyberflow",
            "ACC_CYBERFLOW_URL",
            "http://cyberflow-backend:8010",
            "ACC_CYBERFLOW_API_KEY",
        ),
        connector(
            "gritirt",
            "ACC_GRITIRT_URL",
            "http://gritirt-backend:8030",
            "ACC_GRITIRT_API_KEY",
        ),
        connector(
            "gritjira",
            "ACC_GRITJIRA_URL",
            "http://gritjira-backend:8040",
            "ACC_GRITJIRA_API_KEY",
        ),
    ];

    let jwt_secret = std::env::var("ACC_JWT_SECRET")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            println!(
                "[AUTH] WARNING: ACC_JWT_SECRET is unset; using an insecure development secret."
            );
            "acc-insecure-development-secret-change-me".to_string()
        });

    let app_state = Arc::new(AppState::new(jwt_secret.clone(), connectors));
    gritshield::core::ioc::AutoWire::component((*app_state).clone());

    let public_paths = vec![
        "/health".to_string(),
        "/api/v1/auth/login".to_string(),
        "/api/v1/auth/logout".to_string(),
        "/api/live-telemetry".to_string(),
        "/mcp".to_string(),
    ];

    let router = Router::new()
        .mount_logger(LogLevel::Info)
        .mount_db(shared_db)
        .add_middleware(gritshield::middleware::CorsMiddleware::new(vec![
            "*".to_string(),
        ]))
        .add_middleware(SecurityHeaders::new())
        .add_middleware(AuthMiddleware::new_jwt(
            JwtHandler::new(&jwt_secret),
            public_paths,
            None,
        ))
        // --- health ---
        .route(("/health", HttpMethod::GET, api::health::health))
        // --- auth ---
        .route(("/api/v1/auth/login", HttpMethod::POST, api::auth::login))
        .route(("/api/v1/auth/logout", HttpMethod::POST, api::auth::logout))
        .route(("/api/v1/auth/me", HttpMethod::GET, api::auth::me))
        // --- system telemetry ---
        .route(("/api/v1/system/metrics", HttpMethod::GET, api::system::metrics))
        .route(("/api/v1/system/info", HttpMethod::GET, api::system::info))
        .route(("/api/v1/system/stream", HttpMethod::GET, api::system::stream))
        // --- monitored services ---
        .route(("/api/v1/services", HttpMethod::GET, api::services::list))
        .route(("/api/v1/services", HttpMethod::POST, api::services::create))
        .route(("/api/v1/services/:id", HttpMethod::GET, api::services::get))
        .route(("/api/v1/services/:id", HttpMethod::PUT, api::services::update))
        .route(("/api/v1/services/:id", HttpMethod::DELETE, api::services::delete))
        .route((
            "/api/v1/services/:id/heartbeats",
            HttpMethod::GET,
            api::services::heartbeats,
        ))
        .route(("/api/v1/heartbeats", HttpMethod::GET, api::services::recent_heartbeats))
        // --- users / RBAC ---
        .route(("/api/v1/users", HttpMethod::GET, api::users::list))
        .route(("/api/v1/users", HttpMethod::POST, api::users::create))
        .route(("/api/v1/users/:id", HttpMethod::PUT, api::users::update))
        .route(("/api/v1/users/:id", HttpMethod::DELETE, api::users::delete))
        // --- platform docs ---
        .route(("/api/v1/docs", HttpMethod::GET, api::docs::list))
        .route(("/api/v1/docs", HttpMethod::POST, api::docs::create))
        .route(("/api/v1/docs", HttpMethod::PUT, api::docs::update))
        .route(("/api/v1/docs", HttpMethod::DELETE, api::docs::delete))
        // --- control plane ---
        .route((
            "/api/v1/control/flush-counters",
            HttpMethod::POST,
            api::control::flush_counters,
        ))
        .route((
            "/api/v1/control/emergency-shutdown",
            HttpMethod::POST,
            api::control::emergency_shutdown,
        ))
        .route(("/api/v1/control/block-ip", HttpMethod::POST, api::control::block_ip))
        .route(("/api/v1/control/unblock-ip", HttpMethod::POST, api::control::unblock_ip))
        // --- downstream integrations ---
        .route(("/api/v1/integrations", HttpMethod::GET, api::integrations::list))
        .route((
            "/api/v1/luxshield/health",
            HttpMethod::GET,
            api::integrations::luxshield_health,
        ))
        .route((
            "/api/v1/luxshield/stats",
            HttpMethod::GET,
            api::integrations::luxshield_stats,
        ))
        .route((
            "/api/v1/luxshield/firewall/rules",
            HttpMethod::GET,
            api::integrations::luxshield_rules,
        ))
        .route((
            "/api/v1/luxshield/firewall/rules",
            HttpMethod::POST,
            api::integrations::luxshield_add_rule,
        ))
        .route((
            "/api/v1/luxshield/firewall/rules",
            HttpMethod::DELETE,
            api::integrations::luxshield_remove_rule,
        ))
        .route((
            "/api/v1/cyberflow/health",
            HttpMethod::GET,
            api::integrations::cyberflow_health,
        ))
        .route((
            "/api/v1/cyberflow/logs",
            HttpMethod::POST,
            api::integrations::cyberflow_logs,
        ))
        .route((
            "/api/v1/cyberflow/threats",
            HttpMethod::POST,
            api::integrations::cyberflow_threats,
        ));

    let host = std::env::var("ACC_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("ACC_PORT").unwrap_or_else(|_| "8000".to_string());

    println!("[ACC] admin-command-center listening on http://{host}:{port}");
    println!("[ACC] MCP surface mounted at http://{host}:{port}/mcp");

    ignite(&host, &port, router).await;
}
