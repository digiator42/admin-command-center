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
                nav {
                    div class="brand" { "🛡️ Gritshield App" }
                    a href="/" { "Home" }
                }
                main class="container" {
                    (content)
                }
                footer {
                    p class="text-3xl font-bold underline" { "Crafted safely with Gritshield Web Engine Engine" }
                }
            }
        }
    }
}
