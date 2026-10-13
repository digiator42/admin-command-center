use gritshield::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde_json::{json, Value};

use crate::api::{auth, internal_error};
use crate::models::{monitored_services, service_heartbeats};

fn db(ctx: &RequestContext) -> Option<&sea_orm::DatabaseConnection> {
    ctx.db.as_ref().map(|d| d.as_ref())
}

fn parse_id(ctx: &RequestContext) -> Result<i32, Response> {
    ctx.param("id")
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| Response::json_bad_request(&json!({ "error": "invalid_id" })))
}

/// `GET /api/v1/services` — list every monitored backend.
pub async fn list(ctx: RequestContext) -> Response {
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    match monitored_services::Entity::find()
        .order_by_asc(monitored_services::Column::Id)
        .all(db)
        .await
    {
        Ok(rows) => Response::json_ok(&json!({ "count": rows.len(), "services": rows })),
        Err(e) => internal_error(e),
    }
}

/// `GET /api/v1/services/:id` — fetch one monitored backend.
pub async fn get(ctx: RequestContext) -> Response {
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(res) => return res,
    };

    match monitored_services::Entity::find_by_id(id).one(db).await {
        Ok(Some(service)) => Response::json_ok(&json!({ "service": service })),
        Ok(None) => Response::json_not_found(&json!({ "error": "not_found", "id": id })),
        Err(e) => internal_error(e),
    }
}

/// `POST /api/v1/services` — register a new backend to probe (Operator+).
pub async fn create(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Operator") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    let body: Value = match ctx.json_body().await {
        Some(v) => v,
        None => return Response::json_bad_request(&json!({ "error": "invalid_json" })),
    };

    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let target_url = body
        .get("target_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if name.is_empty() || target_url.is_empty() {
        return Response::json_bad_request(&json!({
            "error": "missing_fields",
            "required": ["name", "target_url"]
        }));
    }

    let expected_status = body.get("expected_status").and_then(|v| v.as_i64()).unwrap_or(200) as i16;
    let ping_interval_seconds = body
        .get("ping_interval_seconds")
        .and_then(|v| v.as_i64())
        .unwrap_or(30) as i32;

    let active = monitored_services::ActiveModel {
        id: NotSet,
        name: Set(name),
        target_url: Set(target_url),
        expected_status: Set(expected_status),
        ping_interval_seconds: Set(ping_interval_seconds),
        current_status: Set("PENDING".to_string()),
        updated_at: NotSet,
    };

    match active.insert(db).await {
        Ok(created) => Response::json_created(&json!({ "service": created })),
        Err(e) => internal_error(e),
    }
}

/// `PUT /api/v1/services/:id` — update a monitored backend (Operator+).
pub async fn update(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Operator") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(res) => return res,
    };

    let body: Value = match ctx.json_body().await {
        Some(v) => v,
        None => return Response::json_bad_request(&json!({ "error": "invalid_json" })),
    };

    let existing = match monitored_services::Entity::find_by_id(id).one(db).await {
        Ok(Some(s)) => s,
        Ok(None) => return Response::json_not_found(&json!({ "error": "not_found", "id": id })),
        Err(e) => return internal_error(e),
    };

    let mut active: monitored_services::ActiveModel = existing.into();
    if let Some(v) = body.get("name").and_then(|v| v.as_str()) {
        active.name = Set(v.to_string());
    }
    if let Some(v) = body.get("target_url").and_then(|v| v.as_str()) {
        active.target_url = Set(v.to_string());
    }
    if let Some(v) = body.get("expected_status").and_then(|v| v.as_i64()) {
        active.expected_status = Set(v as i16);
    }
    if let Some(v) = body.get("ping_interval_seconds").and_then(|v| v.as_i64()) {
        active.ping_interval_seconds = Set(v as i32);
    }

    match active.update(db).await {
        Ok(updated) => Response::json_ok(&json!({ "service": updated })),
        Err(e) => internal_error(e),
    }
}

/// `DELETE /api/v1/services/:id` — remove a monitored backend (Admin).
pub async fn delete(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(res) => return res,
    };

    match monitored_services::Entity::delete_by_id(id).exec(db).await {
        Ok(res) => Response::json_ok(&json!({ "deleted": res.rows_affected, "id": id })),
        Err(e) => internal_error(e),
    }
}

/// `GET /api/v1/services/:id/heartbeats` — recent probe history for one backend.
pub async fn heartbeats(ctx: RequestContext) -> Response {
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match parse_id(&ctx) {
        Ok(id) => id,
        Err(res) => return res,
    };
    let limit = ctx
        .query_param("limit")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(100)
        .min(1000);

    match service_heartbeats::Entity::find()
        .filter(service_heartbeats::Column::ServiceId.eq(id))
        .order_by_desc(service_heartbeats::Column::Id)
        .limit(limit)
        .all(db)
        .await
    {
        Ok(rows) => Response::json_ok(&json!({ "count": rows.len(), "heartbeats": rows })),
        Err(e) => internal_error(e),
    }
}

/// `GET /api/v1/heartbeats` — the newest probes across every backend.
pub async fn recent_heartbeats(ctx: RequestContext) -> Response {
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let limit = ctx
        .query_param("limit")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(100)
        .min(1000);

    match service_heartbeats::Entity::find()
        .order_by_desc(service_heartbeats::Column::Id)
        .limit(limit)
        .all(db)
        .await
    {
        Ok(rows) => Response::json_ok(&json!({ "count": rows.len(), "heartbeats": rows })),
        Err(e) => internal_error(e),
    }
}
