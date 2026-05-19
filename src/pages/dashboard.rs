
use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::{register_page};
use maud::{html};
use std::sync::atomic::Ordering;
use sea_orm::EntityTrait;

use crate::models::monitored_services;

pub async fn handler(ctx: RequestContext) -> Response {
    // Fetch our atomic numbers securely out of the thread pool context
    let active = ctx.telemetry.active_connections.load(Ordering::SeqCst);
    let blocked = ctx.telemetry.total_blocked_ips.load(Ordering::SeqCst);
    let throttled = ctx.telemetry.total_rate_limited_reqs.load(Ordering::SeqCst);

    // Fetch live running services tracking state out of database pool hook
    let mut services_list = Vec::new();
    if let Some(ref db_pool) = ctx.db {
        if let Ok(records) = monitored_services::Entity::find().all(db_pool.as_ref()).await {
            services_list = records;
        }
    }

    // Render the panel body using 100% type-safe Maud code with Tailwind utility styles!
    let panel_body = html! {
        div class="space-y-8" {
            div {
                h1 class="text-3xl font-bold tracking-tight text-slate-100" { "Kernel Engine Telemetry" }
                p class="mt-2 text-sm text-slate-400" { 
                    "Real-time metric streams processed directly out of GritShield asynchronous Tokio pipeline contexts." 
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
                        span class="text-4xl font-extrabold font-mono text-slate-100" { (active) }
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
                        span class="text-4xl font-extrabold font-mono text-slate-100" { (blocked) }
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
                        span class="text-4xl font-extrabold font-mono text-slate-100" { (throttled) }
                        span class="text-xs text-slate-500" { "buckets exhausted" }
                    }
                    div class="absolute bottom-0 left-0 right-0 h-[2px] bg-amber-500" {}
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
        }
    }.into_string();

    render!(raw, "GritShield Control Panel Hub", panel_body)
}

register_page!(HttpMethod::GET, handler);