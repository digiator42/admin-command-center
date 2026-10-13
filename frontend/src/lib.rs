//! ACC console — command-center dashboard as a Veloasm SPA.
//!
//! - JSON login (token in memory only) gating the whole shell
//! - Telemetry WebSocket driving live metric tiles with a 3 s reconnect loop
//! - Monitored-services inventory (REST-synced)
//! - RBAC-gated control deck (flush counters, emergency shutdown) behind
//!   confirm modals instead of `confirm()`/`alert()`

use veloasm::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

mod api;
mod components;
mod net;
mod state;

use crate::components::*;
use crate::state::AppState;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    run_app();
}

pub fn run_app() {
    // Seed global context before mounting so every component can pull the
    // shared state with `use_context::<AppState>()`.
    let app = AppState::new();
    provide!(app.clone());

    let authed = app.auth;
    let api_error = app.api_error;

    let shell = view! {
        <div class="min-h-screen bg-slate-950 text-slate-200 text-sm">
            {
                move || if authed.get().is_some() {
                    view! {
                        <div>
                            <TopBar />
                            <main class="container mx-auto px-4 py-6 space-y-8 max-w-6xl">
                                <MetricsTiles />
                                <ControlDeck />
                                <ServicesTable />
                                <div class="grid gap-4 lg:grid-cols-2">
                                    <IntegrationsPanel />
                                    <UsersPanel />
                                </div>
                                <DocsPanel />
                                <div class="font-mono text-xs text-rose-400">{ api_error }</div>
                            </main>
                            <ToastStack />
                        </div>
                    }
                } else {
                    view! { <LoginView /> }
                }
            }
        </div>
    };
    mount(shell);
}
