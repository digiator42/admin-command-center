use gritshield::{
    core::server::run_server,
    prelude::*,
    security::middleware::{AuthMiddleware, LoggerMiddleware},
};

use crate::workers::monitor;

mod pages {
    pub mod dashboard;
    pub mod auth {
        mod login {
            pub mod index;
            pub mod post;
        }
        mod register {
            pub mod index;
            pub mod post;
        }
    }
}

mod bootstrap;
mod models;
mod root;
mod security;
mod telemetry_ws;
mod workers;

#[get("/static/:*path")]
async fn static_assets(ctx: RequestContext) -> Response {
    let path = ctx.params.get("*path").unwrap().as_str();

    let full_fs_path = format!("static/{}", path);

    Response::static_file(&full_fs_path)
}

#[tokio::main]
async fn main() {
    let shared_db = Arc::new(bootstrap::connect_and_migrate_db().await);

    let monitor_db_pool = std::sync::Arc::clone(&shared_db);
    tokio::spawn(async move {
        println!("[MONITOR] Launching upstream engine health worker thread cluster...");
        monitor::start_service_monitor_loop(monitor_db_pool).await;
    });

    let public_paths = vec![
        "/static/**".to_string(),
        "/auth/**".to_string(),
        "/dashboard".to_string(),
        "/api/live-telemetry".to_string(),
    ];

    let router = Router::new()
        .add_middleware(LoggerMiddleware)
        .add_middleware(AuthMiddleware::new_session(
            public_paths,
            Some("auth/login"),
        ))
        .mound_db(shared_db)
        .mount_file_routes("src/pages")
        .expect("Failed to map file paths tree");

    println!("[GRITSHIELD] Booting engine cluster...");
    run_server("127.0.0.1", "8000", router, true).await;
}
