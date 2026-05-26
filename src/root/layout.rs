use gritshield::prelude::RequestContext;
use maud::html;

pub fn main_layout(title: &str, content: maud::Markup, ctx: &RequestContext) -> maud::Markup {
    // Check if user is authenticated by inspecting the context session
    let is_authenticated = ctx.is_user_authenticated();

    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (title) }
                link rel="stylesheet" href="/static/css/style.css";
                script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
            }
            body {
                nav class="flex items-center gap-6 p-4 bg-slate-900 text-slate-200" {
                    div class="brand font-bold text-lg" { "🛡️ Gritshield App" }
                    a href="/" { "Home" }
                    a href="/dashboard" { "Dashboard" }

                    @if !is_authenticated {
                        a href="/auth/login" { "Login" }
                        a href="/auth/register" { "Register" }
                    } @else {
                        // The developer can also dynamically greet the user if needed!
                        a href="/logout" class="text-red-400 font-medium" { "Logout" }
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
