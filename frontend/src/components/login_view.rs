//! Login view: username + password form driving the JSON login endpoint.
//!
//! On success the token goes into memory-only state (never localStorage)
//! and the dashboard mounts. Failures render inline — no alerts.

use veloasm::prelude::*;

use crate::state::{AppState, Auth};

#[component]
pub fn LoginView() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let username = signal!(String::new());
    let password = signal!(String::new());
    let busy = signal!(false);
    let form_error = app.auth_error;

    let on_submit = {
        let app = app.clone();
        move |_| {
            let app = app.clone();
            if busy.get() {
                return;
            }
            let u = username.get().trim().to_string();
            let p = password.get();
            if u.is_empty() || p.is_empty() {
                form_error.set("username and password are required".to_string());
                return;
            }
            busy.set(true);
            form_error.set(String::new());
            password.set(String::new());
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::login(&u, &p).await {
                    Ok(reply) => {
                        let expires_at_ms = if reply.expires_in > 0 {
                            js_sys::Date::now() as i64 + reply.expires_in * 1000
                        } else {
                            0
                        };
                        app.auth.set(Some(Auth {
                            token: reply.token,
                            user: reply.user,
                            expires_at_ms,
                        }));
                        app.auth_error.set(String::new());
                        crate::net::connect(&app);
                        crate::net::start_services_poll(&app);
                    }
                    Err(e) => {
                        app.auth_error.set(e.to_string());
                    }
                }
                busy.set(false);
            });
        }
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-slate-950 px-4">
            <div class="w-full max-w-sm rounded-xl border border-slate-800 bg-slate-900/60 p-8">
                <div class="text-center">
                    <div class="text-3xl text-indigo-400">"⬢"</div>
                    <h1 class="mt-2 text-xl font-extrabold tracking-wide text-slate-100">"Command Center"</h1>
                    <p class="mt-1 text-sm text-slate-400">"Authenticate into the control kernel platform."</p>
                </div>
                <div class="mt-6 flex flex-col gap-3">
                    <label class="text-xs font-semibold text-slate-400">
                        "Username"
                        <input
                            type="text"
                            class="mt-1 w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
                            placeholder="operator"
                            bind:value={ username }
                        />
                    </label>
                    <label class="text-xs font-semibold text-slate-400">
                        "Password"
                        <input
                            type="password"
                            class="mt-1 w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500"
                            placeholder="••••••••"
                            bind:value={ password }
                        />
                    </label>
                    <div class="text-xs font-mono text-rose-400 min-h-4">{ form_error }</div>
                    <button
                        class="w-full bg-indigo-600 hover:bg-indigo-500 text-white font-bold py-2 px-4 rounded-lg transition-colors"
                        class:opacity-50={ move || busy.get() }
                        on:click={ on_submit }>
                        { move || if busy.get() { "Signing in…" } else { "Sign in" } }
                    </button>
                </div>
            </div>
        </div>
    }
}
