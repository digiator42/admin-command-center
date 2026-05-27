use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use maud::{PreEscaped, html};
use sea_orm::EntityTrait;
use std::sync::atomic::Ordering;

use crate::models::monitored_services;
use crate::security::rbac::RbacExtensions;

pub async fn handler(ctx: RequestContext) -> Response {
    // Fetch our atomic numbers securely out of the thread pool context (Initial load states)
    let active = ctx.telemetry.active_connections.load(Ordering::SeqCst);
    let blocked = ctx.telemetry.total_blocked_ips.load(Ordering::SeqCst);
    let throttled = ctx.telemetry.total_rate_limited_reqs.load(Ordering::SeqCst);

    // Fetch live running services tracking state out of database pool hook
    let mut services_list = Vec::new();
    if let Some(ref db_pool) = ctx.db {
        if let Ok(records) = monitored_services::Entity::find()
            .all(db_pool.as_ref())
            .await
        {
            services_list = records;
        }
    }

    let csrf_token = ctx.get_csrf_token();

    // Render the panel body using 100% type-safe Maud code with Tailwind utility styles!
    let panel_body = html! {
        div class="space-y-8" {
            // Header panel card
            div class="flex items-center justify-between border-b border-slate-800 pb-5" {
                div {
                    h1 class="text-3xl font-bold tracking-tight text-slate-100" { "Kernel Engine Telemetry" }
                    p class="mt-2 text-sm text-slate-400" {
                        "Real-time metric streams processed directly out of GritShield asynchronous Tokio pipeline contexts."
                    }
                }
            }

            div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3" {
                // Active Streams Card
                div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6" {
                    div class="flex items-center justify-between" {
                        span class="text-sm font-medium text-slate-400" { "Active Network Streams" }
                        span class="bg-emerald-400/10 text-emerald-400 px-2 py-1 text-xs font-medium rounded-md" { "Live" }
                    }
                    div class="mt-4 flex items-baseline gap-2" {
                        // ADDED ID FOR WS MAPPING
                        span id="ws-metrics-active" class="text-4xl font-extrabold font-mono text-slate-100 animate-fade-in" { (active) }
                        span class="text-xs text-slate-500" { "concurrent loops" }
                    }
                    div class="absolute bottom-0 left-0 right-0 h-[2px] bg-emerald-500" {}
                }

                // Perimeter Defenses Card
                div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6" {
                    div class="flex items-center justify-between" {
                        span class="text-sm font-medium text-slate-400" { "Perimeter Defenses" }
                        span class="bg-rose-400/10 text-rose-400 px-2 py-1 text-xs font-medium rounded-md" { "Drop Event" }
                    }
                    div class="mt-4 flex items-baseline gap-2" {
                        // ADDED ID FOR WS MAPPING
                        span id="ws-metrics-blocked" class="text-4xl font-extrabold font-mono text-slate-100" { (blocked) }
                        span class="text-xs text-slate-500" { "attacks dropped" }
                    }
                    div class="absolute bottom-0 left-0 right-0 h-[2px] bg-rose-600" {}
                }

                // Throttled Card
                div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6" {
                    div class="flex items-center justify-between" {
                        span class="text-sm font-medium text-slate-400" { "Throttled Allocations" }
                        span class="bg-amber-400/10 text-amber-400 px-2 py-1 text-xs font-medium rounded-md" { "HTTP 429" }
                    }
                    div class="mt-4 flex items-baseline gap-2" {
                        // ADDED ID FOR WS MAPPING
                        span id="ws-metrics-throttled" class="text-4xl font-extrabold font-mono text-slate-100" { (throttled) }
                        span class="text-xs text-slate-500" { "buckets exhausted" }
                    }
                    div class="absolute bottom-0 left-0 right-0 h-[2px] bg-amber-500" {}
                }
            }

           // Conditional Administrative Operator Deck
            div class="p-6 rounded-xl border border-slate-800 bg-slate-900/40 space-y-4" {
                // EMBED THE TOKEN SECURELY IN THE DOM FOR CLIENT EXTRAPOLATION
                input type="hidden" id="global-csrf-token" value=(csrf_token);

                h2 class="text-xl font-bold text-slate-200" { "Core Management Node Controls" }

                @if ctx.has_role("Operator") {
                    p class="text-sm text-slate-400" {
                        "Authorized operational actions available to your active command level profile."
                    }
                    div class="flex flex-wrap gap-4 pt-2" {
                        button id="flush-counters-btn" class="px-4 py-2 bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-sm font-semibold rounded-lg transition-colors disabled:opacity-50" {
                            "Flush Network Telemetry Counters"
                        }
                        @if ctx.has_role("SuperAdmin") {
                            button id="emergency-shutdown-btn" class="px-4 py-2 bg-rose-600/20 hover:bg-rose-600 border border-rose-500/30 text-rose-300 text-sm font-semibold rounded-lg transition-colors disabled:opacity-50" {
                                "Emergency Infrastructure Shutdown"
                            }
                        }
                    }
                } @else {
                    div class="p-4 bg-amber-500/5 border border-amber-500/10 rounded-lg flex items-center gap-3 text-amber-400/80 text-sm" {
                        span class="text-base" { "⚠️" }
                        p { "Your access role profile limits modification privileges." }
                    }
                }
            }

            // Upstream Trackers Subtitle Section
            div {
                h2 class="text-xl font-bold tracking-tight text-slate-200" { "Monitored Platform Ecosystem" }
            }

            // Iterate and render every tracked service in your inventory dynamically!
            @for service in services_list {
                div class="p-6 bg-slate-900 border border-slate-800 rounded-xl flex items-center justify-between" {
                    div {
                        h3 class="text-lg font-bold text-slate-100" { (service.name) }
                        p class="text-xs text-slate-400 font-mono" { (service.target_url) }
                    }

                    span class={
                        "px-3 py-1 rounded-full text-xs font-semibold flex items-center gap-2 "
                        (if service.current_status == "UP" { "bg-green-500/10 text-green-400" }
                        else if service.current_status == "DEGRADED" { "bg-yellow-500/10 text-yellow-400" }
                        else { "bg-red-500/10 text-red-400" })
                    } {
                        span class={
                            "w-2 h-2 rounded-full "
                            (if service.current_status == "UP" { "bg-green-400 animate-pulse" }
                            else if service.current_status == "DEGRADED" { "bg-yellow-400 animate-pulse" }
                            else { "bg-red-400" })
                        } {}
                        (service.current_status)
                    }
                }
            }

            // AUTOMATED LIVE TELEMETRY STREAM SCRIPT LAYER
            script {
                (PreEscaped(r#"
                    const wsScheme = window.location.protocol === 'https:' ? 'wss://' : 'ws://';
                    const wsUrl = `${wsScheme}${window.location.host}/api/live-telemetry`;
                    let ws;

                    console.log("[TELEMETRY STREAM] Initializing secure WebSocket connection to telemetry stream endpoint...");
                    
                    function connectTelemetry() {
                        ws = new WebSocket(wsUrl);
                        
                        ws.onmessage = function(event) {
                            try {
                                const metrics = JSON.parse(event.data);

                                console.debug("[TELEMETRY STREAM] Received live metrics frame:", metrics);
                                
                                // Live updates mapping cleanly to DOM targets without redrawing
                                if(metrics.active_connections !== undefined) {
                                    document.getElementById('ws-metrics-active').innerText = metrics.active_connections;
                                }
                                if(metrics.total_blocked_ips !== undefined) {
                                    document.getElementById('ws-metrics-blocked').innerText = metrics.total_blocked_ips;
                                }
                                if(metrics.total_rate_limited_reqs !== undefined) { 
                                    document.getElementById('ws-metrics-throttled').innerText = metrics.total_rate_limited_reqs;
                                }
                            } catch(e) {
                                console.error("[TELEMETRY STREAM ERROR] Invalid frame mapping parser resolution.", e);
                            }
                        };
                        
                        ws.onclose = function() {
                            console.warn("[TELEMETRY DISCONNECTED] Connection dropped. Re-establishing secure pipe in 3s...");
                            setTimeout(connectTelemetry, 3000); // Intelligent auto-reconnect fallback loop
                        };
                    }
                    
                    // Boot the connection instantly
                    connectTelemetry();

                    // CORE CONTROL NODE FUNCTIONALITIES
                    document.addEventListener("DOMContentLoaded", function() {
                    const flushBtn = document.getElementById("flush-counters-btn");
                    const shutdownBtn = document.getElementById("emergency-shutdown-btn");
                    
                    // READ THE SECURE SERVER TOKEN OUT OF THE HIDDEN INPUT ELEMENT
                    const csrfToken = document.getElementById("global-csrf-token")?.value;

                    if (flushBtn) {
                        flushBtn.addEventListener("click", async function() {
                            if(!confirm("Are you sure you want to clear all active network counters?")) return;
                            
                            flushBtn.disabled = true;
                            try {
                                const res = await fetch("/api/admin/flush-counters", { 
                                    method: "POST",
                                    headers: {
                                        // ATTACH THE CRYPTOGRAPHIC PASS FRAMEWAY FOR VALIDATION HIERARCHIES
                                        "X-CSRF-Token": csrfToken,
                                        "Content-Type": "application/json"
                                    }
                                });
                                const data = await res.json();
                                if (res.ok) {
                                    alert(data.message || "Counters flushed successfully.");
                                } else {
                                    alert("Error: " + (data.error || "Execution failed."));
                                }
                            } catch (err) {
                                alert("Network submission failure tracing connection.");
                            } finally {
                                flushBtn.disabled = false;
                            }
                        });
                    }

                    if (shutdownBtn) {
                        shutdownBtn.addEventListener("click", async function() {
                            if(!confirm("CRITICAL WARNING:\n\nThis will instantly terminate the engine kernel clusters. Proceed?")) return;
                            
                            shutdownBtn.disabled = true;
                            try {
                                const res = await fetch("/api/admin/emergency-shutdown", { 
                                    method: "POST",
                                    headers: {
                                        // ATTACH THE CRYPTOGRAPHIC PASS FRAMEWAY HERE TOO
                                        "X-CSRF-Token": csrfToken,
                                        "Content-Type": "application/json"
                                    }
                                });
                                if (res.ok) {
                                    alert("Shutdown signal sent. Server context offline.");
                                    window.location.reload();
                                } else {
                                    const data = await res.json();
                                    alert("Shutdown rejected: " + (data.error || "Forbidden"));
                                }
                            } catch (err) {
                                alert("Connection lost. Kernel cluster down.");
                            } finally {
                                shutdownBtn.disabled = false;
                            }
                        });
                    }
                });
                "#))
            }
        }
    };

    render!(ctx, "GritShield Control Panel Hub", panel_body)
}

register_page!(HttpMethod::GET, handler);
