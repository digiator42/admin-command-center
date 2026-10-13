//! Live metric tiles driven by the telemetry socket.

use veloasm::prelude::*;

use crate::state::{human_bytes, AppState};

#[component]
pub fn MetricsTiles() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let t = app.telemetry;
    let active = memo!(move || t.get().active_connections.to_string());
    let blocked = memo!(move || t.get().total_blocked_ips.to_string());
    let throttled = memo!(move || t.get().total_rate_limited_reqs.to_string());
    let latency = memo!(move || format!("{} ms", t.get().cpu_worker_latency_ms));
    let memory = memo!(move || human_bytes(t.get().memory_usage_bytes));

    view! {
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-slate-100">"Kernel Engine Telemetry"</h1>
            <p class="mt-2 text-sm text-slate-400">"Real-time metric streams over the telemetry socket."</p>

            <div class="mt-5 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
                <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-slate-400">"Active Network Streams"</span>
                        <span class="bg-emerald-400/10 text-emerald-400 px-2 py-1 text-xs font-medium rounded-md">"Live"</span>
                    </div>
                    <div class="mt-4 flex items-baseline gap-2">
                        <span class="text-4xl font-extrabold font-mono text-slate-100">{ active }</span>
                        <span class="text-xs text-slate-500">"concurrent loops"</span>
                    </div>
                    <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-emerald-500"></div>
                </div>

                <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-slate-400">"Perimeter Defenses"</span>
                        <span class="bg-rose-400/10 text-rose-400 px-2 py-1 text-xs font-medium rounded-md">"Drop Event"</span>
                    </div>
                    <div class="mt-4 flex items-baseline gap-2">
                        <span class="text-4xl font-extrabold font-mono text-slate-100">{ blocked }</span>
                        <span class="text-xs text-slate-500">"attacks dropped"</span>
                    </div>
                    <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-rose-600"></div>
                </div>

                <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-slate-400">"Throttled Allocations"</span>
                        <span class="bg-amber-400/10 text-amber-400 px-2 py-1 text-xs font-medium rounded-md">"HTTP 429"</span>
                    </div>
                    <div class="mt-4 flex items-baseline gap-2">
                        <span class="text-4xl font-extrabold font-mono text-slate-100">{ throttled }</span>
                        <span class="text-xs text-slate-500">"buckets exhausted"</span>
                    </div>
                    <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-amber-500"></div>
                </div>

                <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-slate-400">"Worker Latency"</span>
                        <span class="bg-sky-400/10 text-sky-400 px-2 py-1 text-xs font-medium rounded-md">"Uptime"</span>
                    </div>
                    <div class="mt-4 flex items-baseline gap-2">
                        <span class="text-4xl font-extrabold font-mono text-slate-100">{ latency }</span>
                        <span class="text-xs text-slate-500">"engine uptime"</span>
                    </div>
                    <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-sky-500"></div>
                </div>

                <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-slate-400">"Memory Footprint"</span>
                        <span class="bg-violet-400/10 text-violet-400 px-2 py-1 text-xs font-medium rounded-md">"RSS"</span>
                    </div>
                    <div class="mt-4 flex items-baseline gap-2">
                        <span class="text-4xl font-extrabold font-mono text-slate-100">{ memory }</span>
                        <span class="text-xs text-slate-500">"resident"</span>
                    </div>
                    <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-violet-500"></div>
                </div>
            </div>
        </div>
    }
}
