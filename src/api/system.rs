use std::sync::atomic::Ordering;

use gritshield::http::SseStream;
use gritshield::prelude::*;
use gritshield::security::telemetry::SystemTelemetry;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use serde_json::{json, Value};

use crate::models::monitored_services;
use crate::state;

/// Snapshot the process-wide framework telemetry counters.
pub fn telemetry_snapshot(t: &SystemTelemetry, started_at: i64) -> Value {
    let total = t.total_allowed_reqs.load(Ordering::Relaxed);
    let duration = t.total_duration_micros.load(Ordering::Relaxed);
    let avg_ms = if total > 0 {
        (duration as f64 / total as f64) / 1000.0
    } else {
        0.0
    };

    json!({
        "active_connections": t.active_connections.load(Ordering::Relaxed),
        "total_requests": total,
        "total_blocked_ips": t.total_blocked_ips.load(Ordering::Relaxed),
        "total_rate_limited_requests": t.total_rate_limited_reqs.load(Ordering::Relaxed),
        "total_errors_4xx": t.total_errors_4xx.load(Ordering::Relaxed),
        "total_errors_5xx": t.total_errors_5xx.load(Ordering::Relaxed),
        "avg_latency_ms": avg_ms,
        "uptime_seconds": chrono::Utc::now().timestamp() - started_at,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// `GET /api/v1/system/metrics` — point-in-time telemetry plus service counts.
pub async fn metrics(ctx: RequestContext) -> Response {
    let st = state::resolve();
    let mut value = telemetry_snapshot(&ctx.telemetry, st.started_at);

    if let Some(db) = ctx.db.as_ref() {
        let db = db.as_ref();
        if let Ok(total) = monitored_services::Entity::find().count(db).await {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("services_total".into(), json!(total));
            }
        }
        if let Ok(up) = monitored_services::Entity::find()
            .filter(monitored_services::Column::CurrentStatus.eq("UP"))
            .count(db)
            .await
        {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("services_up".into(), json!(up));
            }
        }
    }

    Response::json_ok(&value)
}

/// `GET /api/v1/system/info` — build/connector metadata for the dashboard.
pub async fn info(ctx: RequestContext) -> Response {
    let st = state::resolve();

    let connectors: Vec<Value> = st
        .connectors
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "url": c.base_url,
                "configured_key": c.api_key.is_some(),
            })
        })
        .collect();

    Response::json_ok(&json!({
        "service": "admin-command-center",
        "version": env!("CARGO_PKG_VERSION"),
        "started_at": st.started_at,
        "uptime_seconds": st.uptime_seconds(),
        "auth": "JWT (HS256) via Authorization: Bearer",
        "claims": crate::api::auth::claims_view(&ctx),
        "connectors": connectors,
    }))
}

/// `GET /api/v1/system/stream` — Server-Sent Events telemetry tail. Emits a
/// `metrics` frame every second, plus an immediate initial frame.
pub async fn stream(ctx: RequestContext) -> Response {
    let st = state::resolve();

    let mut sse = SseStream::new();
    sse.queue_initial("metrics", &telemetry_snapshot(&ctx.telemetry, st.started_at).to_string());

    let pump = sse.clone();
    let telemetry = ctx.telemetry.clone();
    let started_at = st.started_at;
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let snap = telemetry_snapshot(&telemetry, started_at);
            if pump.send_json("metrics", &snap).is_err() {
                break;
            }
        }
    });

    Response::sse(sse)
}
