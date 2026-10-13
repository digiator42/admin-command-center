//! Top bar: brand, role badge, connection state, sign-out.

use veloasm::prelude::*;

use crate::state::AppState;

#[component]
pub fn TopBar() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let conn = app.conn;
    let dot = memo!(move || conn.get().dot().to_string());
    let label = memo!(move || conn.get().label().to_string());
    let role = memo!(move || {
        app.auth
            .get()
            .map(|a| a.user.role.clone())
            .unwrap_or_else(|| "Guest".to_string())
    });
    let name = memo!(move || {
        app.auth
            .get()
            .map(|a| a.user.username.clone())
            .unwrap_or_default()
    });

    view! {
        <header>
            <nav class="flex items-center justify-between p-4 bg-slate-900 text-white font-bold tracking-wide uppercase">
                <div class="text-lg">"⬢ ACC"</div>
                <div class="flex items-center gap-4">
                    <span class="font-mono text-xs text-slate-400 normal-case tracking-normal">{ name }</span>
                    <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                        { role }
                    </span>
                    <span class="flex items-center gap-1.5 text-xs font-normal normal-case tracking-normal text-slate-400">
                        <span class={ dot } style="width:8px;height:8px;border-radius:9999px;display:inline-block;"></span>
                        <span>{ label }</span>
                    </span>
                    <button
                        class="bg-indigo-600 hover:bg-indigo-700 text-white text-xs font-bold tracking-wide py-1.5 px-3 rounded-lg transition-colors"
                        on:click={ move |_| {
                            crate::net::disconnect();
                            app.auth.set(None);
                        } }>
                        "Sign out"
                    </button>
                </div>
            </nav>
        </header>
    }
}
