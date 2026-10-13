//! Monitored services inventory: REST-synced table with add/delete and
//! expandable per-service heartbeat history.

use veloasm::prelude::*;

use crate::state::{AppState, MonitoredService};

#[component]
pub fn ServiceRow(service: MonitoredService) -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");
    let service_id = service.id;
    let expanded = signal!(false);
    let beats = signal_vec(Vec::<crate::api::Heartbeat>::new());

    let badge = class_names!(
        "px-3 py-1 rounded-full text-xs font-semibold flex items-center gap-2",
        service.badge()
    );
    let dot = class_names!("w-2 h-2 rounded-full", service.dot());
    let name = service.name.clone();
    let url = service.target_url.clone();
    let status = service.current_status.clone();

    let can_admin = memo!(move || {
        app.auth.get().map(|a| a.user.is_admin()).unwrap_or(false)
    });

    let app_toggle = app.clone();
    let app_delete = app.clone();

    let on_toggle = {
        let app = app_toggle.clone();
        let beats = beats.clone();
        move |_| {
            let app = app.clone();
            let beats = beats.clone();
            let open = !expanded.get();
            expanded.set(open);
            if open {
                let Some(token) = app.auth.get().map(|a| a.token) else {
                    return;
                };
                wasm_bindgen_futures::spawn_local(async move {
                    match crate::api::service_heartbeats(&token, service_id, 20).await {
                        Ok(history) => {
                            app.api_error.set(String::new());
                            beats.with_mut(|v| *v = history);
                        }
                        Err(e) => {
                            if e.is_unauthorized() {
                                app.auth.set(None);
                            } else {
                                app.api_error.set(e.to_string());
                            }
                        }
                    }
                });
            }
        }
    };

    let on_delete = {
        let app = app_delete.clone();
        move |_| {
            let app = app.clone();
            let Some(token) = app.auth.get().map(|a| a.token) else {
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::delete_service(&token, service_id).await {
                    Ok(()) => {
                        app.api_error.set(String::new());
                        crate::net::sync_services(&app);
                    }
                    Err(e) => {
                        if e.is_unauthorized() {
                            app.auth.set(None);
                        } else {
                            app.api_error.set(e.to_string());
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="bg-slate-900 border border-slate-800 rounded-xl overflow-hidden">
            <div class="p-6 flex items-center justify-between gap-3">
                <button class="flex-1 min-w-0 flex items-center justify-between gap-3 text-left" on:click={ on_toggle }>
                    <div class="min-w-0">
                        <h3 class="text-lg font-bold text-slate-100 truncate">{ name }</h3>
                        <p class="text-xs text-slate-400 font-mono truncate">{ url }</p>
                    </div>
                    <span class={ badge }>
                        <span class={ dot }></span>
                        { status }
                    </span>
                </button>
                <button
                    class="shrink-0 bg-transparent border-none text-slate-600 text-lg cursor-pointer px-1 hover:text-rose-400"
                    style:display={ move || if can_admin.get() { "block".to_string() } else { "none".to_string() } }
                    title="delete service"
                    on:click={ on_delete }>
                    "×"
                </button>
            </div>
            <div
                class="border-t border-slate-800 px-6 py-3"
                style:display={ move || if expanded.get() { "block".to_string() } else { "none".to_string() } }
            >
                <div class="text-[11px] font-bold tracking-[2px] text-slate-500 mb-2">"LAST 20 HEARTBEATS"</div>
                <div class="flex items-end gap-1 h-12">
                    {
                        for b in beats key = |b: &crate::api::Heartbeat| b.id {
                            <HeartbeatBar beat={ b.clone() } />
                        }
                    }
                </div>
            </div>
        </div>
    }
}

/// One heartbeat as a latency bar (height ~ log scale, color by status).
#[component]
pub fn HeartbeatBar(beat: crate::api::Heartbeat) -> DomNode {
    let h = (8.0 + (beat.latency_ms.max(1) as f64).ln() * 6.0).min(48.0) as u32;
    let bar = if beat.http_status_code >= 200 && beat.http_status_code < 300 {
        format!("width:8px;height:{h}px;border-radius:2px;background:#34d399;")
    } else {
        format!("width:8px;height:{h}px;border-radius:2px;background:#fb7185;")
    };
    let title = format!(
        "{} · {} ms · HTTP {}",
        beat.checked_at, beat.latency_ms, beat.http_status_code
    );
    view! {
        <div style={ bar } title={ title }></div>
    }
}

#[component]
pub fn ServicesTable() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");
    let list = app.services.clone();
    let count_list = app.services.clone();
    let empty_list = app.services.clone();
    let count = memo!(move || format!("{} tracked", count_list.len()));
    let is_empty = memo!(move || empty_list.len() == 0);

    let can_operate = memo!(move || {
        app.auth.get().map(|a| a.user.is_operator()).unwrap_or(false)
    });
    let name_input = signal!(String::new());
    let url_input = signal!(String::new());
    let form_error = signal!(String::new());

    let on_create = {
        let app = app.clone();
        move |_| {
            let app = app.clone();
            let name = name_input.get().trim().to_string();
            let target = url_input.get().trim().to_string();
            if name.is_empty() || target.is_empty() {
                form_error.set("name and target_url are required".to_string());
                return;
            }
            form_error.set(String::new());
            wasm_bindgen_futures::spawn_local(async move {
                let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                match crate::api::create_service(&token, &name, &target).await {
                    Ok(_) => {
                        name_input.set(String::new());
                        url_input.set(String::new());
                        app.api_error.set(String::new());
                        crate::net::sync_services(&app);
                    }
                    Err(e) => {
                        if e.is_unauthorized() {
                            app.auth.set(None);
                        } else {
                            form_error.set(e.to_string());
                        }
                    }
                }
            });
        }
    };

    view! {
        <div>
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-bold tracking-tight text-slate-200">"Monitored Platform Ecosystem"</h2>
                <span class="font-mono text-[11px] text-slate-500">{ count }</span>
            </div>
            <div
                class="mt-3 p-3 rounded-xl border border-slate-800 bg-slate-900/40 flex-col gap-2"
                style:display={ move || if can_operate.get() { "flex".to_string() } else { "none".to_string() } }
            >
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="service name…"
                        bind:value={ name_input }
                    />
                    <input
                        type="text"
                        class="flex-[2] min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="http://svc:9000/health"
                        bind:value={ url_input }
                    />
                    <button
                        class="bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-bold px-4 py-1.5 rounded-lg whitespace-nowrap"
                        on:click={ on_create }>
                        "Add"
                    </button>
                </div>
                <div class="text-[11px] font-mono text-rose-400">{ form_error }</div>
            </div>
            <div class="mt-3 flex flex-col gap-3">
                {
                    move || if is_empty.get() {
                        view! { <div class="px-3 py-8 text-center text-sm text-slate-500">"No services registered yet."</div> }
                    } else {
                        DomNode::empty()
                    }
                }
                {
                    for s in list key = |s: &MonitoredService| s.id {
                        <ServiceRow service={ s.clone() } />
                    }
                }
            </div>
        </div>
    }
}
