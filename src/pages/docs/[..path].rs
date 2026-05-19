use gritshield::protocol::request::HttpMethod;
use gritshield::protocol::response::Response;
use gritshield::render;
use gritshield::routing::trie::RequestContext;
use maud::{Markup, html};
use std::fs;
use std::path::PathBuf;

pub async fn handler(ctx: RequestContext) -> Response {
    let doc_subpath = match ctx.params.get("*path") {
        Some(untrusted_val) => untrusted_val.as_str(),
        None => "index",
    };

    println!("[DEBUG] Extracted Router Params Matrix: {:?}", ctx.params);

    // Attempt File System Read
    let mut target_file = PathBuf::from("docs_content");
    for segment in doc_subpath.split('/') {
        target_file.push(segment);
    }
    if target_file.is_dir() {
        target_file.push("index.md");
    } else if target_file.extension().is_none() {
        target_file.set_extension("md");
    }

    let page_content: Markup = match fs::read_to_string(&target_file) {
        Ok(raw_markdown) => {
            // File exists on disk! Render it cleanly.
            render_doc_body("Disk Asset", &raw_markdown)
        }
        Err(_) => {
            if let Some(ref db_pool) = ctx.db {
                // Pull the native &PgPool out of Sea-ORM's abstraction layer
                let raw_sqlx_pool = db_pool.get_postgres_connection_pool();

                let db_result = sqlx::query!(
                    "SELECT title, markdown_body FROM platform_docs WHERE slug = $1",
                    doc_subpath
                )
                // Pass the native SQLx pool reference right here!
                .fetch_optional(raw_sqlx_pool)
                .await;

                match db_result {
                    Ok(Some(record)) => render_doc_body(&record.title, &record.markdown_body),
                    _ => render_not_found(doc_subpath),
                }
            } else {
                render_not_found(doc_subpath)
            }
        }
    };

    render!("GritShield Knowledge Base", page_content)
}

// Clean, reusable Maud helper components
fn render_doc_body(title: &str, body: &str) -> Markup {
    html! {
        article class="space-y-4" {
            h1 class="text-2xl font-bold border-b border-slate-800 pb-4 text-slate-100" { (title) }
            div class="p-6 bg-slate-900/40 border border-slate-800 rounded-xl" {
                p class="font-mono text-sm text-slate-300 whitespace-pre-wrap leading-relaxed" { (body) }
            }
        }
    }
}

fn render_not_found(slug: &str) -> Markup {
    html! {
        div class="p-8 text-center border border-dashed border-slate-800 rounded-xl max-w-xl mx-auto my-12" {
            h1 class="text-lg font-semibold text-slate-300" { "Documentation Route Missing" }
            p class="text-xs text-slate-500 mt-2" { "Could not find resource tracking context on disk or inside platform_docs database." }
            div class="mt-4 px-3 py-1 bg-slate-900 rounded font-mono text-[11px] text-slate-400 inline-block" {
                "Target Slug: " (slug)
            }
        }
    }
}

gritshield::register_page!(HttpMethod::GET, handler);
