use futures_util::sink::SinkExt;
use gritshield::{
    deps::tokio_tungstenite::WebSocketStream,
    deps::tokio_tungstenite::tungstenite::Message,
    deps::tokio::net::TcpStream, 
    deps::serde::Serialize,
    routing::trie::RequestContext,
};

use std::sync::atomic::Ordering;
use std::time::Duration;

#[derive(Serialize)]
struct LiveMetricsPayload {
    active_connections: u64,
    total_blocked_ips: u64,
    cpu_worker_latency_ms: u128,
    memory_usage_bytes: u64,
}

pub async fn telemetry_stream_handler(
    mut ws_stream: WebSocketStream<TcpStream>,
    ctx: RequestContext,
) {
    println!("[ACC TELEMETRY] Live Monitoring Operator Connected!");

    // Simple performance loop monitoring state
    let context_start_time = ctx.start_time;

    loop {
        // 1. Query the live metrics out of their atomic synchronization cells
        let metrics = LiveMetricsPayload {
            active_connections: ctx.telemetry.active_connections.load(Ordering::SeqCst),
            total_blocked_ips: ctx.telemetry.total_blocked_ips.load(Ordering::SeqCst),
            cpu_worker_latency_ms: context_start_time.elapsed().as_millis(),
            memory_usage_bytes: sysinfo_memory_lookup(), // Abstracted system query helper
        };

        // Serialize to standard JSON string format
        if let Ok(json_string) = serde_json::to_string(&metrics) {
            if ws_stream.send(Message::Text(json_string)).await.is_err() {
                println!("[ACC TELEMETRY] Target pipeline connection dropped.");
                break;
            }
        }

        // 4. Sleep for 1 second before streaming the next heartbeat metrics packet
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn sysinfo_memory_lookup() -> u64 {
    // Return mock calculations or integrate sysinfo crate properties here
    42 * 1024 * 1024
}

// Maps this async logic directly to "ws://localhost:8080/api/live-telemetry"
gritshield::register_ws!("/api/live-telemetry", telemetry_stream_handler);
