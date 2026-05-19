use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::{register_page, routing::templates::get_template};
use maud::{PreEscaped, html};

use std::fs;
use std::path::PathBuf;

use crate::root::layout::main_layout;

pub async fn handler(ctx: RequestContext) -> Response {
    // Extract the wildcard catch-all segments parsed by GritShield's file router.
    // If the folder is named [..path].rs, your router populates ctx.params.get("path")
    let doc_subpath = match ctx.params.get("path") {
        Some(untrusted_val) => untrusted_val.as_str(), // Clean wrapper handling
        None => "index", // Fallback to landing docs page if root /docs is hit
    };

    // Resolve the path on disk inside the local project workspace
    let mut target_file = PathBuf::from("docs_content");
    for segment in doc_subpath.split('/') {
        target_file.push(segment);
    }
    // If it's a folder or simple file, check for a markdown or text resource extension
    if target_file.is_dir() {
        target_file.push("index.md");
    } else if target_file.extension().is_none() {
        target_file.set_extension("md");
    }

    // Try reading the documentation file asset off disk
    let page_content = match fs::read_to_string(&target_file) {
        Ok(raw_markdown) => {
            // For now, let's treat it as a pristine documentation body block.
            html! {
                article class="prose prose-invert max-w-none" {
                    h1 class="text-2xl font-bold border-b border-slate-800 pb-4 text-slate-100" {
                        "Documentation Resource"
                    }
                    p class="mt-4 font-mono text-sm text-slate-400 whitespace-pre-wrap" { (raw_markdown) }
                }
            }
        }
        Err(_) => {

            if let Some(ref db_pool) = ctx.db {
                // let db_doc = sqlx::query!("SELECT content FROM docs WHERE slug = $1", doc_subpath)...
            }

            html! {
                div class="p-8 text-center border border-dashed border-slate-800 rounded-xl" {
                    h1 class="text-lg font-semibold text-slate-300" { "Documentation Page Not Found" }
                    p class="text-xs text-slate-500 mt-1" { "The requested path target could not be verified on disk or database." }
                    a href="/docs" class="mt-4 inline-block text-xs bg-slate-800 text-slate-200 px-3 py-1.5 rounded hover:bg-slate-700 transition" {
                        "Back to Core Index"
                    }
                }
            }
        }
    };

    // Wrap the computed component layout straight into your beautiful framework wrapper!
    let rendered_view = main_layout("GritShield Docs Suite", page_content);

    Response::new(200, Sanitizer::trust(&rendered_view.into_string()))
}

// Register using your unified HTTP GET routing method hook
register_page!(HttpMethod::GET, handler);
