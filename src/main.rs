use gritshield::{core::server::run_server, prelude::*, security::db::connect};

mod root;

#[get("/")]
async fn index(_ctx: RequestContext) -> Response {
    render!(
        "Welcome Home",
        html! {
            h1 { "Victory!" }
            p { "Your application is successfully running under the Gritshield kernel." }
        }
    )
}

#[get("/static/:*path")]
async fn static_assets(ctx: RequestContext) -> Response {
    let path = ctx.params.get("*path").unwrap().as_str();
    Response::static_file(path)
}

#[tokio::main]
async fn main() {
    // Standard SQLite connection
    let db = connect("postgres://postgres:admin@localhost:5432/acc_db")
        .await
        .unwrap();
    let shared_db = Arc::new(db);

    let router = Router::new().mound_db(shared_db);

    println!("[GRITSHIELD] Booting engine cluster...");
    run_server("127.0.0.1", "8080", router, true).await;
}
