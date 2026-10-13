//! Platform docs: metadata list, reader, create + delete.

use veloasm::prelude::*;

use crate::state::AppState;

fn refresh_docs(app: &AppState) {
    let Some(token) = app.auth.get().map(|a| a.token) else {
        return;
    };
    let app = app.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match crate::api::list_docs(&token).await {
            Ok(docs) => {
                app.api_error.set(String::new());
                app.docs.with_mut(|v| *v = docs);
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
pub fn DocRow(doc: crate::api::DocMeta) -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");
    let row_slug = doc.slug.clone();
    let title = if doc.title.is_empty() {
        doc.slug.clone()
    } else {
        doc.title.clone()
    };

    let can_admin = memo!(move || {
        app.auth.get().map(|a| a.user.is_admin()).unwrap_or(false)
    });
    let app_open = app.clone();

    let on_open = {
        let app = app_open.clone();
        let slug = row_slug.clone();
        move |_| { let app = app.clone(); let slug = slug.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                match crate::api::get_doc(&token, &slug).await {
                    Ok(full) => {
                        app.api_error.set(String::new());
                        app.doc_body.set(full.markdown_body);
                        app.open_doc.set(Some(full.slug));
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

    let on_delete = {
        let app = app.clone();
        let slug = row_slug.clone();
        move |_| { let app = app.clone(); let slug = slug.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                match crate::api::delete_doc(&token, &slug).await {
                    Ok(()) => {
                        app.api_error.set(String::new());
                        if app.open_doc.get().as_deref() == Some(slug.as_str()) {
                            app.open_doc.set(None);
                            app.doc_body.set(String::new());
                        }
                        refresh_docs(&app);
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
        <div class="flex items-center justify-between gap-2">
            <button
                class="flex-1 min-w-0 text-left px-3 py-2 rounded-lg bg-slate-950 border border-slate-800 hover:border-indigo-500/50"
                on:click={ on_open }>
                <div class="text-[13px] font-semibold text-slate-200 truncate">{ title }</div>
                <div class="font-mono text-[11px] text-slate-500 truncate">{ doc.slug.clone() }</div>
            </button>
            <button
                class="shrink-0 bg-transparent border-none text-slate-600 text-base cursor-pointer px-1 hover:text-rose-400"
                style:display={ move || if can_admin.get() { "block".to_string() } else { "none".to_string() } }
                title="delete doc"
                on:click={ on_delete }>
                "×"
            </button>
        </div>
    }
}

#[component]
pub fn DocsPanel() -> DomNode {
    let app = use_context::<AppState>().expect("AppState must be provided before mounting");

    let list = app.docs.clone();
    let open_slug = app.open_doc.clone();
    let body = app.doc_body;

    let can_write = memo!(move || {
        app.auth.get().map(|a| a.user.is_operator()).unwrap_or(false)
    });

    let slug_input = signal!(String::new());
    let title_input = signal!(String::new());
    let body_input = signal!(String::new());
    let form_error = signal!(String::new());

    let on_create = {
        let app = app.clone();
        move |_| {
            let app = app.clone();
            let slug = slug_input.get().trim().to_string();
            let title = title_input.get().trim().to_string();
            let md = body_input.get();
            if slug.is_empty() {
                form_error.set("slug is required".to_string());
                return;
            }
            form_error.set(String::new());
            wasm_bindgen_futures::spawn_local(async move {
                let Some(token) = app.auth.get().map(|a| a.token) else { return; };
                match crate::api::create_doc(&token, &slug, &title, &md).await {
                    Ok(_) => {
                        slug_input.set(String::new());
                        title_input.set(String::new());
                        body_input.set(String::new());
                        app.api_error.set(String::new());
                        refresh_docs(&app);
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

    refresh_docs(&app);

    view! {
        <section class="bg-slate-900/40 border border-slate-800 rounded-xl p-4">
            <h2 class="text-xs font-bold tracking-[2px] text-slate-500 mb-2">"PLATFORM DOCS"</h2>
            <div class="flex flex-col gap-2 mb-3">
                {
                    for d in list key = |d: &crate::api::DocMeta| d.id {
                        <DocRow doc={ d.clone() } />
                    }
                }
            </div>
            {
                move || if open_slug.get().is_some() {
                    view! {
                        <pre class="scrollthin whitespace-pre-wrap break-words font-mono text-xs text-slate-300 bg-slate-950 border border-slate-800 rounded-lg p-3 max-h-64 overflow-y-auto mb-3">{ body }</pre>
                    }
                } else {
                    DomNode::empty()
                }
            }
            <div
                class="flex-col gap-2 border-t border-slate-800 pt-3"
                style:display={ move || if can_write.get() { "flex".to_string() } else { "none".to_string() } }
            >
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="slug e.g. runbooks/onboarding…"
                        bind:value={ slug_input }
                    />
                    <input
                        type="text"
                        class="flex-1 min-w-0 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
                        placeholder="title…"
                        bind:value={ title_input }
                    />
                </div>
                <textarea
                    class="scrollthin bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 font-mono text-[11px] text-slate-300 h-20 focus:outline-none focus:border-indigo-500"
                    bind:value={ body_input }
                ></textarea>
                <div>
                    <button
                        class="bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-bold px-4 py-1.5 rounded-lg"
                        on:click={ on_create }>
                        "Save doc"
                    </button>
                </div>
                <div class="text-[11px] font-mono text-rose-400">{ form_error }</div>
            </div>
        </section>
    }
}

