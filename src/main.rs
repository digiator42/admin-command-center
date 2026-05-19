use gritshield::{
    core::server::run_server,
    prelude::*,
    security::{db::connect, middleware::LoggerMiddleware},
};


mod pages {
    pub mod dashboard;
    #[path = "docs/[..path].rs"]
    pub mod docs_wildcard;
}

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

    let full_fs_path = format!("static/{}", path);

    Response::static_file(&full_fs_path)
}

#[tokio::main]
async fn main() {
    // Standard SQLite connection
    let db = connect("postgres://postgres:admin@localhost:5432/acc_db")
        .await
        .unwrap();
    let shared_db = Arc::new(db);

    let router = Router::new()
        .add_middleware(LoggerMiddleware)
        .mound_db(shared_db)
        .mount_file_routes("src/pages")
        .expect("Failed to map file paths tree");

    println!("[GRITSHIELD] Booting engine cluster...");
    run_server("127.0.0.1", "8080", router, true).await;
}
