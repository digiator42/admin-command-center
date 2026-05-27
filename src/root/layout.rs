use gritshield::prelude::RequestContext;
use maud::html;

pub fn main_layout(title: &str, content: maud::Markup, ctx: &RequestContext) -> maud::Markup {
    // Check if user is authenticated by inspecting the context session
    let is_authenticated = ctx.is_user_authenticated();
    // Capture user profile parameters smoothly from current context
    let current_role = ctx.get_user_role().unwrap_or("Guest".to_string());

    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (title) }
                link rel="stylesheet" href="/static/css/style.css";
                script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
            }
            body {
                nav class="flex items-center justify-between mb-8 p-4 bg-slate-900 text-white font-bold tracking-wide uppercase" {                    // Left-aligned logo
                    div class="brand font-bold text-lg" { "🛡️ Gritshield App" }

                    // Centered navigation links
                    div class="flex-1 flex justify-center gap-6" {
                        a href="/" { "Home" }
                        a href="/dashboard" { "Dashboard" }

                        @if !is_authenticated {
                            a href="/auth/login" { "Login" }
                            a href="/auth/register" { "Register" }
                        } @else {
                            form method="POST" action="/logout" {
                                button type="submit" class="-mt-6 bg-indigo-600 hover:bg-indigo-700 text-white font-bold tracking-wide py-2 px-4 rounded-lg transition-colors" {
                                    "Sign out"
                                }
                            }
                        }
                    }

                    // Right-aligned badge
                    div class="flex items-center gap-4" {
                        span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20" {
                            (current_role)
                        }
                    }
                }

                main class="container" {
                    (content)
                }

                footer {
                    p class="text-xs font-bold underline" {
                        "Crafted safely with Gritshield Web Engine"
                    }
                }
            }
        }
    }
}
