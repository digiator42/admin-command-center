//! Monitored services inventory (REST-synced, heartbeat statuses).

use veloasm::prelude::*;

use crate::state::{AppState, MonitoredService};

#[component]
pub fn ServiceRow(service: MonitoredService) -> DomNode {
    let badge = class_names!(
        "px-3 py-1 rounded-full text-xs font-semibold flex items-center gap-2",
        service.badge()
    );
    let dot = class_names!("w-2 h-2 rounded-full", service.dot());
    let name = service.name.clone();
    let url = service.target_url.clone();
    let status = service.current_status.clone();

    view! {
        <div class="p-6 bg-slate-900 border border-slate-800 rounded-xl flex items-center justify-between">
            <div class="min-w-0">
                <h3 class="text-lg font-bold text-slate-100 truncate">{ name }</h3>
                <p class="text-xs text-slate-400 font-mono truncate">{ url }</p>
            </div>
            <span class={ badge }>
                <span class={ dot }></span>
                { status }
            </span>
        </div>
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

    view! {
        <div>
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-bold tracking-tight text-slate-200">"Monitored Platform Ecosystem"</h2>
                <span class="font-mono text-[11px] text-slate-500">{ count }</span>
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
