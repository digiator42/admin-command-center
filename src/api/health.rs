use gritshield::prelude::*;
use serde_json::json;

use crate::state;

/// `GET /health` — public liveness probe with a database reachability check.
pub async fn health(ctx: RequestContext) -> Response {
    let st = state::resolve();

    let db_ok = match ctx.db.as_ref() {
        Some(db) => db.ping().await.is_ok(),
        None => false,
    };

    Response::json_ok(&json!({
        "service": "admin-command-center",
        "status": if db_ok { "ok" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "db": db_ok,
        "uptime_seconds": st.uptime_seconds(),
    }))
}
