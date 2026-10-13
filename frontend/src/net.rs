//! Live telemetry: a `WebSocket` to `/api/live-telemetry` with a 3 s
//! reconnect loop (mirrors the current vanilla client), plus the services
//! inventory sync.
//!
//! Frames are 1/sec JSON: `{active_connections, total_blocked_ips,
//! cpu_worker_latency_ms, memory_usage_bytes, total_rate_limited_reqs}`.
//! The socket takes `?token=` (headers are impossible on `new WebSocket`).

use std::cell::RefCell;

use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::api;
use crate::state::{AppState, ConnStatus, Telemetry};

thread_local! {
    /// The live socket, if any. `WebSocket` is not `Clone`.
    static SOCKET: RefCell<Option<web_sys::WebSocket>> = const { RefCell::new(None) };
    /// Reconnect timer guard. `true` while the app wants the socket up.
    static WANTED: RefCell<bool> = const { RefCell::new(false) };
}

/// Connect with the current session token (no-op when already connected).
pub fn connect(app: &AppState) {
    WANTED.with(|w| *w.borrow_mut() = true);
    if SOCKET.with(|s| s.borrow().is_some()) {
        return;
    }
    open_socket(app);
}

fn open_socket(app: &AppState) {
    close_socket_only();
    app.conn.set(ConnStatus::Connecting);

    let token = app.auth.get().map(|a| a.token).unwrap_or_default();
    let url = api::telemetry_url(&token);
    let ws = match web_sys::WebSocket::new(&url) {
        Ok(ws) => ws,
        Err(err) => {
            web_sys::console::error_1(&format!("WS open failed: {err:?}").into());
            app.conn.set(ConnStatus::Closed);
            schedule_reconnect(app);
            return;
        }
    };

    // --- frames ------------------------------------------------------------
    let on_message = {
        let app = app.clone();
        Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let Some(text) = event.data().as_string() else {
                return;
            };
            match serde_json::from_str::<Telemetry>(&text) {
                Ok(t) => {
                    app.conn.set(ConnStatus::Open);
                    app.telemetry.set(t);
                }
                Err(err) => {
                    web_sys::console::warn_1(
                        &format!("skipping malformed telemetry frame: {err}").into(),
                    )
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>)
    };
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    // --- errors surface state; close schedules the 3 s reconnect ------------
    let on_error = {
        let conn = app.conn;
        Closure::wrap(Box::new(move |_: web_sys::ErrorEvent| {
            conn.set(ConnStatus::Reconnecting);
        }) as Box<dyn FnMut(web_sys::ErrorEvent)>)
    };
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    let on_close = {
        let app = app.clone();
        Closure::wrap(Box::new(move |_: web_sys::CloseEvent| {
            app.conn.set(ConnStatus::Reconnecting);
            close_socket_only();
            schedule_reconnect(&app);
        }) as Box<dyn FnMut(web_sys::CloseEvent)>)
    };
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();

    SOCKET.with(|slot| *slot.borrow_mut() = Some(ws));
}

fn schedule_reconnect(app: &AppState) {
    let app = app.clone();
    wasm_bindgen_futures::spawn_local(async move {
        TimeoutFuture::new(3_000).await;
        let wanted = WANTED.with(|w| *w.borrow());
        if wanted {
            open_socket(&app);
        }
    });
}

fn close_socket_only() {
    SOCKET.with(|slot| {
        if let Some(ws) = slot.borrow_mut().take() {
            let _ = ws.close();
        }
    });
}

/// Hang up and stop reconnecting (logout).
pub fn disconnect() {
    WANTED.with(|w| *w.borrow_mut() = false);
    close_socket_only();
}

/// Pull the services inventory (called on login + on a slow poll).
pub fn sync_services(app: &AppState) {
    let Some(token) = app.auth.get().map(|a| a.token) else {
        return;
    };
    let app = app.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match api::list_services(&token).await {
            Ok(services) => {
                app.api_error.set(String::new());
                app.services.with_mut(|v| *v = services);
            }
            Err(e) => {
                app.api_error.set(e.to_string());
                if e.is_unauthorized() {
                    app.auth.set(None);
                }
            }
        }
    });
}

/// Poll services every 30 s (heartbeats move slowly; WS covers the fast path).
pub fn start_services_poll(app: &AppState) {
    sync_services(app);
    let app = app.clone();
    let _interval = gloo_timers::callback::Interval::new(30_000, move || {
        sync_services(&app);
    });
    std::mem::forget(_interval);
}
