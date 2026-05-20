use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use maud::html;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};

use crate::models::system_users;
use crate::security::crypto::hash_password;

// Render the HTML form (GET /register)
pub async fn get_handler(ctx: RequestContext) -> Response {
    // Flexing GritShield's automatic CSRF token tracking
    let csrf_token = ctx.session.as_ref().and_then(|s| {
        s.lock().unwrap().data.get("csrf_token").cloned()
    }).unwrap_or_default();

    let body = html! {
        div class="min-h-screen flex items-center justify-center bg-slate-950 px-4" {
            div class="max-w-md w-full bg-slate-900 border border-slate-800 p-8 rounded-xl space-y-6" {
                div class="text-center" {
                    h1 class="text-2xl font-bold text-slate-100" { "Create ACC Node Identity" }
                    p class="text-sm text-slate-400 mt-1" { "Provision a new operator profile inside the control plane." }
                }
                
                form method="POST" action="/register" class="space-y-4" {
                    // Native GritShield CSRF protection shield injection
                    input type="hidden" name="csrf_token" value=(csrf_token);

                    div {
                        label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2" { "Username" }
                        input type="text" name="username" required class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500" {}
                    }
                    div {
                        label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2" { "Password" }
                        input type="password" name="password" required class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500" {}
                    }
                    div {
                        label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2" { "Assigned Role" }
                        select name="role" class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500" {
                            option value="Auditor" { "Auditor (Read-Only)" }
                            option value="Operator" { "Operator (Standard Command)" }
                            option value="SuperAdmin" { "SuperAdmin" }
                        }
                    }
                    button type="submit" class="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded-lg transition-colors pt-3" {
                        "Register System Account"
                    }
                }
            }
        }
    }.into_string();

    render!(raw, "ACC | Provision Node", body)
}

// Process submission parameters (POST /register)
pub async fn post_handler(ctx: RequestContext) -> Response {
    let db = match ctx.db {
        Some(ref pool) => pool,
        None => return Response::new(500, Sanitizer::trust("Database context pool missing".into())),
    };

    // Grab parsed form fields via GritShield's underlying extractor
    let form_data = ctx.req.parse_form_body();
    let username = form_data.fields.get("username").map(|s| s.as_str()).unwrap_or("");
    let password = form_data.fields.get("password").map(|s| s.as_str()).unwrap_or("");
    let role = form_data.fields.get("role").map(|s| s.as_str()).unwrap_or("Auditor");

    if username.is_empty() || password.is_empty() {
        return Response::new(400, Sanitizer::trust("Missing required creation credentials".into()));
    }

    // Secure cryptographic hashing verification sequence
    let hashed = match hash_password(password) {
        Ok(h) => h,
        Err(_) => return Response::new(500, Sanitizer::trust("Internal crypto fault".into())),
    };

    let new_user = system_users::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        username: Set(username.to_string()),
        password_hash: Set(hashed),
        role: Set(role.to_string()),
        created_at: Set(None),
    };

    match new_user.insert(db.as_ref()).await {
        Ok(_) => {
            // Creation success! Redirect our system operator immediately back to login page path
            let mut res = Response::new(303, Sanitizer::trust("Redirecting...".into()));
            res.headers.push(("Location".to_string(), "/login".to_string()));
            res
        }
        Err(err) => {
            Response::new(400, Sanitizer::trust(format!("Account deployment rejected: User might already exist ({})", err).as_str()))
        }
    }
}

// register_page!(HttpMethod::GET, get_handler);
// register_page!(HttpMethod::POST, post_handler);