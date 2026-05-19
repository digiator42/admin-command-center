use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::{register_page, routing::templates::get_template};
use maud::{PreEscaped, html};
use std::sync::atomic::Ordering;

pub async fn handler(ctx: RequestContext) -> Response {
    // Fetch our atomic numbers securely out of the thread pool context
    let active = ctx.telemetry.active_connections.load(Ordering::SeqCst);
    let blocked = ctx.telemetry.total_blocked_ips.load(Ordering::SeqCst);
    let throttled = ctx.telemetry.total_rate_limited_reqs.load(Ordering::SeqCst);

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
        }
    }.into_string();

    render!(raw, "GritShield Control Panel Hub", panel_body)
}

register_page!(HttpMethod::GET, handler);
