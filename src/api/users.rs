use gritshield::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, EntityTrait, QueryOrder, Set};
use serde_json::{json, Value};

use crate::api::{auth, internal_error};
use crate::models::system_users;
use crate::security::crypto;

fn db(ctx: &RequestContext) -> Option<&sea_orm::DatabaseConnection> {
    ctx.db.as_ref().map(|d| d.as_ref())
}

fn valid_role(role: &str) -> bool {
    matches!(role, "Admin" | "Operator" | "Auditor")
}

/// `GET /api/v1/users` — list operator accounts (Admin).
pub async fn list(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    match system_users::Entity::find()
        .order_by_asc(system_users::Column::Id)
        .all(db)
        .await
    {
        Ok(rows) => {
            let users: Vec<Value> = rows.iter().map(auth::user_view).collect();
            Response::json_ok(&json!({ "count": users.len(), "users": users }))
        }
        Err(e) => internal_error(e),
    }
}

/// `POST /api/v1/users` — create an operator account (Admin).
pub async fn create(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    let body: Value = match ctx.json_body().await {
        Some(v) => v,
        None => return Response::json_bad_request(&json!({ "error": "invalid_json" })),
    };

    let username = body.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let password = body.get("password").and_then(|v| v.as_str()).unwrap_or("");
    let role = body.get("role").and_then(|v| v.as_str()).unwrap_or("Auditor").to_string();

    if username.is_empty() || password.is_empty() {
        return Response::json_bad_request(&json!({
            "error": "missing_fields",
            "required": ["username", "password"]
        }));
    }
    if !valid_role(&role) {
        return Response::json_bad_request(&json!({
            "error": "invalid_role",
            "allowed": ["Admin", "Operator", "Auditor"]
        }));
    }

    let password_hash = match crypto::generate_hash(&password) {
        Ok(h) => h,
        Err(e) => return internal_error(e),
    };

    let active = system_users::ActiveModel {
        id: NotSet,
        username: Set(username),
        password_hash: Set(password_hash),
        role: Set(role),
        created_at: NotSet,
    };

    match active.insert(db).await {
        Ok(created) => Response::json_created(&json!({ "user": auth::user_view(&created) })),
        Err(e) => {
            if e.to_string().to_lowercase().contains("unique") {
                Response::json_conflict(&json!({ "error": "username_taken" }))
            } else {
                internal_error(e)
            }
        }
    }
}

/// `PUT /api/v1/users/:id` — change an account's role and/or password (Admin).
pub async fn update(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match ctx
        .param("id")
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| Response::json_bad_request(&json!({ "error": "invalid_id" })))
    {
        Ok(id) => id,
        Err(res) => return res,
    };

    let body: Value = match ctx.json_body().await {
        Some(v) => v,
        None => return Response::json_bad_request(&json!({ "error": "invalid_json" })),
    };

    let existing = match system_users::Entity::find_by_id(id).one(db).await {
        Ok(Some(u)) => u,
        Ok(None) => return Response::json_not_found(&json!({ "error": "not_found", "id": id })),
        Err(e) => return internal_error(e),
    };

    let mut active: system_users::ActiveModel = existing.into();

    if let Some(role) = body.get("role").and_then(|v| v.as_str()) {
        if !valid_role(role) {
            return Response::json_bad_request(&json!({
                "error": "invalid_role",
                "allowed": ["Admin", "Operator", "Auditor"]
            }));
        }
        active.role = Set(role.to_string());
    }

    if let Some(password) = body.get("password").and_then(|v| v.as_str()) {
        if password.len() < 6 {
            return Response::json_bad_request(&json!({
                "error": "weak_password",
                "detail": "password must be at least 6 characters"
            }));
        }
        match crypto::generate_hash(password) {
            Ok(h) => active.password_hash = Set(h),
            Err(e) => return internal_error(e),
        }
    }

    match active.update(db).await {
        Ok(updated) => Response::json_ok(&json!({ "user": auth::user_view(&updated) })),
        Err(e) => internal_error(e),
    }
}

/// `DELETE /api/v1/users/:id` — remove an operator account (Admin).
pub async fn delete(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };
    let id = match ctx
        .param("id")
        .and_then(|v| v.parse::<i32>().ok())
        .ok_or_else(|| Response::json_bad_request(&json!({ "error": "invalid_id" })))
    {
        Ok(id) => id,
        Err(res) => return res,
    };

    match system_users::Entity::delete_by_id(id).exec(db).await {
        Ok(res) => Response::json_ok(&json!({ "deleted": res.rows_affected, "id": id })),
        Err(e) => internal_error(e),
    }
}
