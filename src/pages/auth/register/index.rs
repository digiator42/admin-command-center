use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use maud::html;

// Render the HTML form (GET /register)
pub async fn get_handler(ctx: RequestContext) -> Response {

    if ctx.is_user_authenticated() {
        // User is logged in, redirect them away from the login page
        return Response::redirect(303, "/dashboard");
    }

    let body = html! {
        div class="min-h-screen flex items-center justify-center bg-slate-950 px-4" {
            div class="max-w-md w-full bg-slate-900 border border-slate-800 p-8 rounded-xl space-y-6" {
                div class="text-center" {
                    h1 class="text-2xl font-bold text-slate-100" { "Create ACC Node Identity" }
                    p class="text-sm text-slate-400 mt-1" { "Provision a new operator profile inside the control plane." }
                }
                
                form method="POST" action="/auth/register/post" class="space-y-4" {
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
    };

    render!(ctx, "ACC | Provision Node", body)
}

register_page!(HttpMethod::GET, get_handler);
