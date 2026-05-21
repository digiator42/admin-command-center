use gritshield::prelude::*;

pub fn main_layout(title: &str, content: maud::Markup) -> maud::Markup {
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
                    a href="/login" { "Login" }
                }
                main class="container" {
                    (content)
                }
                footer {
                    p class="text-3xl font-bold underline" {
                        "Crafted safely with Gritshield Web Engine"
                    }
                }
            }
        }
    }
}
