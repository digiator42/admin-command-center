use std::time::Instant;

use gritshield::prelude::*;
use reqwest::Method;
use serde_json::{json, Value};

use crate::state;

/// Generic reverse-proxy helper: forwards a request to a configured connector
/// and relays the upstream status + JSON body back to the caller.
async fn proxy(ctx: &RequestContext, connector: &str, method: Method, path: &str) -> Response {
    let st = state::resolve();

    let Some(conn) = st.connector(connector) else {
        return Response::json_service_unavailable(&json!({
            "error": "connector_not_configured",
            "connector": connector
        }));
    };

    let url = conn.url(path);
    let mut req = st.http.request(method, &url);
    if let Some(key) = &conn.api_key {
        req = req.header("X-API-Key", key);
    }
    if !ctx.raw_body.is_empty() {
        let content_type = ctx
            .header("content-type")
            .unwrap_or("application/json")
            .to_string();
        req = req
            .header(reqwest::header::CONTENT_TYPE, content_type)
            .body(ctx.raw_body.clone());
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            let value: Value =
                serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            let mut out = Response::json_ok(&value);
            out.status = status;
            out
        }
        Err(e) => Response::json_service_unavailable(&json!({
            "error": "upstream_unreachable",
            "connector": connector,
            "detail": e.to_string()
        })),
    }
}

/// `GET /api/v1/integrations` — probe every downstream product's `/health`.
pub async fn list(_ctx: RequestContext) -> Response {
    let st = state::resolve();
    let mut items: Vec<Value> = Vec::with_capacity(st.connectors.len());

    for c in st.connectors.iter() {
        let started = Instant::now();
        let mut req = st.http.get(c.url("/health"));
        if let Some(key) = &c.api_key {
            req = req.header("X-API-Key", key);
        }

        let (reachable, status, health) = match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();
                let value: Value =
                    serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
                (status == 200, status, value)
            }
            Err(e) => (false, 0, json!({ "error": e.to_string() })),
        };

        items.push(json!({
            "name": c.name,
            "url": c.base_url,
            "reachable": reachable,
            "status_code": status,
            "latency_ms": started.elapsed().as_millis(),
            "health": health,
        }));
    }

    Response::json_ok(&json!({ "integrations": items }))
}

/// `GET /api/v1/luxshield/health`
pub async fn luxshield_health(ctx: RequestContext) -> Response {
    proxy(&ctx, "luxshield", Method::GET, "/health").await
}

/// `GET /api/v1/luxshield/stats`
pub async fn luxshield_stats(ctx: RequestContext) -> Response {
    proxy(&ctx, "luxshield", Method::GET, "/api/v1/network/stats").await
}

/// `GET /api/v1/luxshield/firewall/rules`
pub async fn luxshield_rules(ctx: RequestContext) -> Response {
    proxy(&ctx, "luxshield", Method::GET, "/api/v1/firewall/rules").await
}

/// `POST /api/v1/luxshield/firewall/rules`
pub async fn luxshield_add_rule(ctx: RequestContext) -> Response {
    proxy(&ctx, "luxshield", Method::POST, "/api/v1/firewall/rules").await
}

/// `DELETE /api/v1/luxshield/firewall/rules?ip=...`
pub async fn luxshield_remove_rule(ctx: RequestContext) -> Response {
    let Some(ip) = ctx.query_param("ip").map(|s| s.to_string()) else {
        return Response::json_bad_request(&json!({ "error": "missing_ip" }));
    };
    if ip.is_empty() {
        return Response::json_bad_request(&json!({ "error": "missing_ip" }));
    }
    let path = format!("/api/v1/firewall/rules?ip={ip}");
    proxy(&ctx, "luxshield", Method::DELETE, &path).await
}

/// `GET /api/v1/cyberflow/health`
pub async fn cyberflow_health(ctx: RequestContext) -> Response {
    proxy(&ctx, "cyberflow", Method::GET, "/health").await
}

/// `POST /api/v1/cyberflow/logs`
pub async fn cyberflow_logs(ctx: RequestContext) -> Response {
    proxy(&ctx, "cyberflow", Method::POST, "/api/v1/logs").await
}

/// `POST /api/v1/cyberflow/threats`
pub async fn cyberflow_threats(ctx: RequestContext) -> Response {
    proxy(&ctx, "cyberflow", Method::POST, "/api/v1/threats").await
}
