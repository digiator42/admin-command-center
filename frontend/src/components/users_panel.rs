//! User management (Admin only): roster list, create form, delete.

use veloasm::prelude::*;

use crate::state::AppState;

fn refresh_users(app: &AppState) {
    let Some(token) = app.auth.get().map(|a| a.token) else {
        return;
    };
    let app = app.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match crate::api::list_users(&token).await {
            Ok(users) => {
                app.api_error.set(String::new());
                app.users.with_mut(|v| *v = users);
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

#[component]
pub fn UserRow(user: crate::api::ManagedUser) -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");
    let user_id = user.id;
    let name = user.username.clone();

    let role_badge = match user.role.as_str() {
        "Admin" => "bg-rose-500/10 text-rose-300",
        "Operator" => "bg-indigo-500/10 text-indigo-300",
        _ => "bg-slate-500/10 text-slate-400",
    };

    view! {
        <div class="flex items-center justify-between gap-2 px-3 py-2 rounded-lg bg-slate-950 border border-slate-800">
            <div class="min-w-0">
                <span class="text-[13px] font-semibold text-slate-200">{ name }</span>
                <span class={ class_names!("ml-2 text-[10px] font-bold px-1.5 py-px rounded", role_badge) }>{ user.role.clone() }</span>
            </div>
            <button
                class="shrink-0 bg-transparent border-none text-slate-600 text-base cursor-pointer px-1 hover:text-rose-400"
                title="delete user"
                on:click={ move |_| {
                    let app = app.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                        match crate::api::delete_user(&token, user_id).await {
                            Ok(()) => {
                                app.api_error.set(String::new());
                                refresh_users(&app);
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
                } }>
                "×"
            </button>
        </div>
    }
}

#[component]
pub fn UsersPanel() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let is_admin = memo!(move || {
        app.auth.get().map(|a| a.user.is_admin()).unwrap_or(false)
    });

    // Admin-only panel; everyone else sees nothing.
    if !is_admin.get() {
        return view! { <div></div> };
    }

    let list = app.users.clone();
    let username_input = signal!(String::new());
    let password_input = signal!(String::new());
    let role_input = signal!("Auditor".to_string());
    let form_error = signal!(String::new());

    refresh_users(&app);

    view! {
        <section class="bg-slate-900/40 border border-slate-800 rounded-xl p-4">
            <h2 class="text-xs font-bold tracking-[2px] text-slate-500 mb-2">"USERS"</h2>
            <div class="flex flex-col gap-2 mb-3">
                {
                    for u in list key = |u: &crate::api::ManagedUser| u.id {
                        <UserRow user={ u.clone() } />
                    }
                }
            </div>
            <div class="flex flex-col gap-2 border-t border-slate-800 pt-3">
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="username…"
                        bind:value={ username_input }
                    />
                    <input
                        type="password"
                        class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="password…"
                        bind:value={ password_input }
                    />
                </div>
                <div class="flex gap-2">
                    <select
                        class="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-2 py-1.5 text-xs text-slate-300"
                        bind:value={ role_input }
                    >
                        <option value="Auditor">"Auditor (read-only)"</option>
                        <option value="Operator">"Operator"</option>
                        <option value="Admin">"Admin"</option>
                    </select>
                    <button
                        class="bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-bold px-4 py-1.5 rounded-lg whitespace-nowrap"
                        on:click={ move |_| {
                            let username = username_input.get().trim().to_string();
                            let password = password_input.get();
                            let role = role_input.get();
                            if username.is_empty() || password.is_empty() {
                                form_error.set("username and password are required".to_string());
                                return;
                            }
                            form_error.set(String::new());
                            let app = app.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                                match crate::api::create_user(&token, &username, &password, &role).await {
                                    Ok(_) => {
                                        username_input.set(String::new());
                                        password_input.set(String::new());
                                        app.api_error.set(String::new());
                                        refresh_users(&app);
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
                        "Add user"
                    </button>
                </div>
                <div class="text-[11px] font-mono text-rose-400">{ form_error }</div>
            </div>
        </section>
    }
}
