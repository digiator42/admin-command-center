use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use maud::html;

// Render Login Form (GET /login)
pub async fn get_handler(ctx: RequestContext) -> Response {
    if ctx.is_user_authenticated() {
        // User is logged in, redirect them away from the login page
        return Response::redirect(303, "/dashboard");
    }

    let error = ctx.get_query_param_decoded("error");

    let csrf_token = ctx.get_session_data("csrf_token");
    println!("[GET /login] CSRF Token for form: {:?}", csrf_token);

    let body = html! {
        div class="min-h-screen flex items-center justify-center bg-slate-950 px-4" {
            div class="max-w-md w-full bg-slate-900 border border-slate-800 p-8 rounded-xl space-y-6" {
                div class="text-center" {
                    h1 class="text-2xl font-bold text-slate-100" { "Admin Command Center" }
                    p class="text-sm text-slate-400 mt-1" { "Authenticate into the core control kernel platform." }
                }

                form method="POST" action="/auth/login/post" class="space-y-4" {
                    div {
                        label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2" { "Username" }
                        input type="text" name="username" required class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500" {}
                    }
                    div {
                        label class="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-2" { "Password" }
                        input type="password" name="password" required class="w-full bg-slate-950 border border-slate-800 rounded-lg px-4 py-2 text-slate-200 focus:outline-none focus:border-indigo-500" {}
                    }
                    button type="submit" class="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-2 px-4 rounded-lg transition-colors pt-3" {
                        "Establish Command Session"
                    }
                }
                @if let Some(error) = error {
                    div class="alert-box error text-center text-red-500 text-sm mt-4" {
                        p { (Sanitizer::trust(&error)) }
                    }
                }
            }
        }
    };

    render!(ctx, "ACC | Secure Core Gateway", body)
}

register_page!(HttpMethod::GET, get_handler);
