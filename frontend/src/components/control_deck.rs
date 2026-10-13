//! Control deck: RBAC-gated operator actions behind confirm modals.
//!
//! Replaces the legacy `confirm()`/`alert()` flow: Operator sees flush,
//! Admin additionally sees emergency shutdown. Results surface as toasts.

use gloo_timers::future::TimeoutFuture;
use veloasm::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::{AppState, TOAST_KEEP, TOAST_TTL_MS};

/// Which destructive action awaits confirmation (`None` = no modal).
#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    Flush,
    Shutdown,
}

fn push_toast(app: &AppState, title: String, detail: String, bad: bool) {
    let id = app.next_toast_id();
    app.toasts.with_mut(|v| {
        v.push(crate::state::Toast {
            id,
            title,
            detail,
            bad,
        });
        while v.len() > TOAST_KEEP {
            v.remove(0);
        }
    });

    let toasts = app.toasts.clone();
    wasm_bindgen_futures::spawn_local(async move {
        TimeoutFuture::new(TOAST_TTL_MS).await;
        toasts.with_mut(|v| {
            if let Some(pos) = v.iter().position(|t| t.id == id) {
                v.remove(pos);
            }
        });
    });
}

#[component]
pub fn ControlDeck() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let role = memo!(move || {
        app.auth
            .get()
            .map(|a| a.user.role.clone())
            .unwrap_or_default()
    });
    let can_operate = memo!(move || {
        app.auth.get().map(|a| a.user.is_operator()).unwrap_or(false)
    });
    let can_admin = memo!(move || {
        app.auth.get().map(|a| a.user.is_admin()).unwrap_or(false)
    });
    let pending = signal!(Option::<PendingAction>::None);
    let busy = signal!(false);

    view! {
        <div class="p-6 rounded-xl border border-slate-800 bg-slate-900/40 space-y-4">
            <h2 class="text-xl font-bold text-slate-200">"Core Management Node Controls"</h2>
            {
                move || if can_operate.get() {
                    view! {
                        <div>
                            <p class="text-sm text-slate-400">"Authorized operational actions available to your command level."</p>
                            <FirewallForm />
                            <div class="flex flex-wrap gap-4 pt-2">
                                <button
                                    class="px-4 py-2 bg-slate-800 hover:bg-slate-700 border border-slate-700 text-slate-200 text-sm font-semibold rounded-lg transition-colors"
                                    on:click={ move |_| pending.set(Some(PendingAction::Flush)) }>
                                    "Flush Network Telemetry Counters"
                                </button>
                                {
                                    move || if can_admin.get() {
                                        view! {
                                            <button
                                                class="px-4 py-2 bg-rose-600/20 hover:bg-rose-600 border border-rose-500/30 text-rose-300 text-sm font-semibold rounded-lg transition-colors"
                                                on:click={ move |_| pending.set(Some(PendingAction::Shutdown)) }>
                                                "Emergency Infrastructure Shutdown"
                                            </button>
                                        }
                                    } else {
                                        DomNode::empty()
                                    }
                                }
                            </div>
                        </div>
                    }
                } else {
                    view! {
                        <div class="p-4 bg-amber-500/5 border border-amber-500/10 rounded-lg flex items-center gap-3 text-amber-400/80 text-sm">
                            <p>{ format!("Your access role ({}) limits modification privileges.", role.get()) }</p>
                        </div>
                    }
                }
            }
            <ConfirmModal pending={ pending } busy={ busy } />
        </div>
    }
}

/// LuxShield firewall proxy: block / unblock an IP through ACC.
#[component]
pub fn FirewallForm() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let ip_input = signal!(String::new());
    let reason_input = signal!(String::new());
    let form_error = signal!(String::new());
    let form_ok = signal!(String::new());

    // One state clone per button; signals are `Copy` and move freely.
    let app_block = app.clone();
    let app_unblock = app.clone();

    view! {
        <div class="flex flex-col gap-2 rounded-lg border border-slate-800 bg-slate-950/60 p-3">
            <div class="text-[11px] font-bold tracking-[2px] text-slate-500">"LUXSHIELD FIREWALL PROXY"</div>
            <div class="flex gap-2">
                <input
                    type="text"
                    class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
                    placeholder="203.0.113.66"
                    bind:value={ ip_input }
                />
                <input
                    type="text"
                    class="flex-[2] min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                    placeholder="reason…"
                    bind:value={ reason_input }
                />
            </div>
            <div class="flex gap-2">
                <button
                    class="bg-rose-600/20 hover:bg-rose-600 border border-rose-500/30 text-rose-300 text-xs font-semibold px-3 py-1.5 rounded-lg"
                    on:click={ move |_| {
                        let app = app_block.clone();
                        let ip = ip_input.get().trim().to_string();
                        let reason = reason_input.get().trim().to_string();
                        if ip.is_empty() {
                            form_error.set("IP is required".to_string());
                            return;
                        }
                        form_error.set(String::new());
                        form_ok.set(String::new());
                        wasm_bindgen_futures::spawn_local(async move {
                            let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                            match crate::api::block_ip(&token, &ip, &reason).await {
                                Ok(()) => {
                                    app.api_error.set(String::new());
                                    form_ok.set(format!("{ip} blocked"));
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
                    } }>
                    "Block IP"
                </button>
                <button
                    class="bg-slate-800 border border-slate-700 rounded-lg px-3 py-1.5 text-xs font-semibold text-slate-300 hover:border-indigo-500"
                    on:click={ move |_| {
                        let app = app_unblock.clone();
                        let ip = ip_input.get().trim().to_string();
                        if ip.is_empty() {
                            form_error.set("IP is required".to_string());
                            return;
                        }
                        form_error.set(String::new());
                        form_ok.set(String::new());
                        wasm_bindgen_futures::spawn_local(async move {
                            let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                            match crate::api::unblock_ip(&token, &ip).await {
                                Ok(()) => {
                                    app.api_error.set(String::new());
                                    form_ok.set(format!("{ip} unblocked"));
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
                    } }>
                    "Unblock IP"
                </button>
                <span class="text-[11px] font-mono text-emerald-400 self-center">{ form_ok }</span>
                <span class="text-[11px] font-mono text-rose-400 self-center">{ form_error }</span>
            </div>
        </div>
    }
}

/// Confirm/cancel modal for the pending destructive action.
#[component]
pub fn ConfirmModal(pending: RwSignal<Option<PendingAction>>, busy: RwSignal<bool>) -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let on_confirm = {
        let app = app.clone();
        move |_| {
            let action = pending.get();
            pending.set(None);
            let Some(action) = action else { return };
            if busy.get() {
                return;
            }
            let app = app.clone();
            let Some(token) = app.auth.get().map(|a| a.token) else {
                return;
            };
            busy.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let message = match action {
                    PendingAction::Flush => crate::api::flush_counters(&token).await,
                    PendingAction::Shutdown => crate::api::emergency_shutdown(&token).await,
                };
                match message {
                    Ok(message) => {
                        app.api_error.set(String::new());
                        push_toast(&app, "Command accepted".to_string(), message, false);
                    }
                    Err(e) => {
                        if e.is_unauthorized() {
                            app.auth.set(None);
                        } else {
                            push_toast(&app, "Command failed".to_string(), e.to_string(), true);
                        }
                    }
                }
                busy.set(false);
            });
        }
    };

    let title = memo!(move || match pending.get() {
        Some(PendingAction::Flush) => "Clear all active network counters?",
        Some(PendingAction::Shutdown) => "CRITICAL: terminate the engine kernel clusters?",
        None => "",
    });
    let danger = memo!(move || pending.get() == Some(PendingAction::Shutdown));

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80"
            style:display={ move || if pending.get().is_some() { "flex".to_string() } else { "none".to_string() } }
            on:click={ move |_| pending.set(None) }
        >
            <div
                class="w-full max-w-md rounded-xl border border-slate-700 bg-slate-900 p-6"
                on:click={ move |ev| {
                    let ev = ev.dyn_ref::<web_sys::Event>().unwrap();
                    ev.stop_propagation();
                } }
            >
                <h3 class="text-lg font-bold text-slate-100">{ title }</h3>
                <p class="mt-1 text-sm text-slate-400">"This action is recorded in the audit trail."</p>
                <div class="mt-5 flex justify-end gap-3">
                    <button
                        class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 text-sm font-semibold rounded-lg"
                        on:click={ move |_| pending.set(None) }>
                        "Cancel"
                    </button>
                    <button
                        class={ if danger.get() { "px-4 py-2 bg-rose-600 hover:bg-rose-500 text-white text-sm font-bold rounded-lg" } else { "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-bold rounded-lg" } }
                        class:opacity-50={ move || busy.get() }
                        on:click={ on_confirm }>
                        { move || if busy.get() { "Working…" } else { "Confirm" } }
                    </button>
                </div>
            </div>
        </div>
    }
}
