//! Downstream integrations: reachability + latency of the sibling
//! platforms (luxshield, cyberflow, gritirt, gritjira), polled slowly.

use veloasm::prelude::*;

use crate::state::AppState;

fn refresh_integrations(app: &AppState) {
    let Some(token) = app.auth.get().map(|a| a.token) else {
        return;
    };
    let app = app.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match crate::api::integrations(&token).await {
            Ok(items) => {
                app.api_error.set(String::new());
                app.integrations.with_mut(|v| *v = items);
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

fn start_integrations_poll(app: AppState) {
    refresh_integrations(&app);
    let _interval =
        gloo_timers::callback::Interval::new(30_000, move || refresh_integrations(&app));
    std::mem::forget(_interval);
}

#[component]
pub fn IntegrationRow(item: crate::api::ConnectorInfo) -> DomNode {
    let dot = if item.reachable {
        "w-2 h-2 rounded-full bg-emerald-400"
    } else {
        "w-2 h-2 rounded-full bg-rose-500"
    };
    let name = item.name.clone();
    let url = item.url.clone();
    let meta = format!("{} · HTTP {}", item.latency_ms, item.status_code);

    view! {
        <div class="flex items-center justify-between gap-3 px-3 py-2 rounded-lg bg-slate-950 border border-slate-800">
            <div class="min-w-0 flex items-center gap-2.5">
                <span class={ dot }></span>
                <div class="min-w-0">
                    <div class="text-[13px] font-semibold text-slate-200 truncate">{ name }</div>
                    <div class="font-mono text-[11px] text-slate-500 truncate">{ url }</div>
                </div>
            </div>
            <span class="font-mono text-[11px] text-slate-500 whitespace-nowrap">{ meta }</span>
        </div>
    }
}

#[component]
pub fn IntegrationsPanel() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");
    let list = app.integrations.clone();
    let empty_list = app.integrations.clone();
    let is_empty = memo!(move || empty_list.len() == 0);

    start_integrations_poll(app.clone());

    view! {
        <section class="bg-slate-900/40 border border-slate-800 rounded-xl p-4">
            <h2 class="text-xs font-bold tracking-[2px] text-slate-500 mb-2">"DOWNSTREAM INTEGRATIONS"</h2>
            {
                move || if is_empty.get() {
                    view! { <div class="text-xs text-slate-600">"Probing connectors…"</div> }
                } else {
                    DomNode::empty()
                }
            }
            <div class="flex flex-col gap-2">
                {
                    for c in list key = |c: &crate::api::ConnectorInfo| c.name.clone() {
                        <IntegrationRow item={ c.clone() } />
                    }
                }
            </div>
        </section>
    }
}
