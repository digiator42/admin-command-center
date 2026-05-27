use gritshield::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

// use crate::security::rbac::RbacExtensions;

#[post("/api/admin/flush-counters", role = "Operator")]
/// Flush the metrics collection atoms cleanly back to baseline states
pub async fn flush_counters_handler(ctx: RequestContext) -> Response {

    // Reset runtime telemetry registers
    ctx.telemetry.active_connections.store(0, Ordering::SeqCst);
    ctx.telemetry.total_blocked_ips.store(0, Ordering::SeqCst);
    ctx.telemetry
        .total_rate_limited_reqs
        .store(0, Ordering::SeqCst);

    println!("[ADMIN COMMAND] Telemetry metric registers cleared cleanly by operator.");

    // Utilizes your new polymorphic `Response::ok` builder mapping clean JSON to the client!
    Response::ok(&HashMap::from([
        ("status", "success"),
        ("message", "Telemetry counters reset successfully."),
    ]))
}

#[post("/api/admin/emergency-shutdown", role = "Admin")]
/// Execute a graceful cascading process termination hook across the cluster
pub async fn emergency_shutdown_handler(_: RequestContext) -> Response {

    println!("[SYSTEM CRITICAL] EMERGENCY SHUTDOWN COMMAND ISSUED BY ADMIN.");

    // Spawn a delayed thread task so the HTTP response frame can ship successfully back to the client first
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        println!("[SYSTEM CRITICAL] Tearing down application container contexts now.");
        std::process::exit(0);
    });

    Response::ok(&HashMap::from([
        ("status", "success"),
        ("message", "Emergency shutdown sequence initialized."),
    ]))
}
