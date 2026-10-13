use std::sync::atomic::Ordering;

use gritshield::prelude::*;
use serde_json::json;

use crate::api::{auth, integrations};

/// `POST /api/v1/control/flush-counters` — zero the framework telemetry
/// registers (Operator+).
pub async fn flush_counters(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Operator") {
        return res;
    }

    let t = &ctx.telemetry;
    t.active_connections.store(0, Ordering::Relaxed);
    t.total_blocked_ips.store(0, Ordering::Relaxed);
    t.total_rate_limited_reqs.store(0, Ordering::Relaxed);
    t.total_allowed_reqs.store(0, Ordering::Relaxed);
    t.total_errors_4xx.store(0, Ordering::Relaxed);
    t.total_errors_5xx.store(0, Ordering::Relaxed);
    t.total_duration_micros.store(0, Ordering::Relaxed);

    Response::json_ok(&json!({
        "status": "ok",
        "message": "telemetry counters reset to baseline"
    }))
}

/// `POST /api/v1/control/emergency-shutdown` — terminate the process shortly
/// after the response is flushed (Admin).
pub async fn emergency_shutdown(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }

    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        println!("[SYSTEM CRITICAL] Tearing down application container contexts now.");
        std::process::exit(0);
    });

    Response::json_accepted(&json!({
        "status": "ok",
        "message": "emergency shutdown sequence initialized"
    }))
}

/// `POST /api/v1/control/block-ip` — convenience proxy that pushes a firewall
/// rule into LuxShield (Operator+).
pub async fn block_ip(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Operator") {
        return res;
    }
    integrations::luxshield_add_rule(ctx).await
}

/// `POST /api/v1/control/unblock-ip` — remove a LuxShield firewall rule
/// (Operator+).
pub async fn unblock_ip(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Operator") {
        return res;
    }
    integrations::luxshield_remove_rule(ctx).await
}
