use gritshield::prelude::*;
use gritshield::security::jwt::{Claims, JwtHandler};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{json, Value};

use crate::api::internal_error;
use crate::models::system_users;
use crate::security::crypto;
use crate::state;

/// Access-token lifetime, in seconds (12 hours).
pub const TOKEN_TTL_SECS: u64 = 12 * 60 * 60;

/// Build a JWT handler bound to the process-wide signing secret.
pub fn jwt_handler() -> JwtHandler {
    JwtHandler::new(&state::resolve().jwt_secret)
}

/// Serialize a user row for API responses, never exposing the password hash.
pub fn user_view(u: &system_users::Model) -> Value {
    json!({
        "id": u.id,
        "username": u.username,
        "role": u.role,
        "created_at": u.created_at,
    })
}

/// Serialize the currently verified JWT claims (or `null` when anonymous).
pub fn claims_view(ctx: &RequestContext) -> Value {
    match &ctx.claims {
        Some(c) => json!({ "sub": c.sub, "role": c.role, "exp": c.exp }),
        None => Value::Null,
    }
}

/// Guard a handler behind the fixed Admin > Operator > Auditor hierarchy.
/// Returns `Some(forbidden_response)` when the caller lacks the role.
pub fn require_role(ctx: &RequestContext, required: &str) -> Option<Response> {
    if ctx.has_fixed_role(required) {
        return None;
    }
    Some(Response::json_forbidden(&json!({
        "error": "forbidden",
        "detail": format!("requires role '{required}' or higher"),
        "required_role": required,
        "your_role": ctx.get_user_role().unwrap_or_else(|| "anonymous".to_string()),
    })))
}

/// `POST /api/v1/auth/login` — verify credentials and mint a signed JWT.
///
/// Body: `{ "username": "...", "password": "..." }`
pub async fn login(ctx: RequestContext) -> Response {
    let Some(db) = ctx.db.as_ref().map(|d| d.as_ref()) else {
        return internal_error("database unavailable");
    };

    let body: Value = match ctx.json_body().await {
        Some(v) => v,
        None => return Response::json_bad_request(&json!({"error": "invalid_json"})),
    };

    let username = body
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let password = body
        .get("password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Response::json_bad_request(&json!({
            "error": "missing_credentials",
            "required": ["username", "password"]
        }));
    }

    let user = match system_users::Entity::find()
        .filter(system_users::Column::Username.eq(username.clone()))
        .one(db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Response::json_unauthorized(&json!({ "error": "invalid_credentials" }))
        }
        Err(e) => return internal_error(e),
    };

    if !crypto::verify_password(&password, &user.password_hash) {
        return Response::json_unauthorized(&json!({ "error": "invalid_credentials" }));
    }

    let claims = Claims::new(user.id.to_string(), user.role.clone(), TOKEN_TTL_SECS);
    let token = match jwt_handler().sign(&claims) {
        Ok(t) => t,
        Err(e) => return internal_error(e),
    };

    Response::json_ok(&json!({
        "token": token,
        "token_type": "Bearer",
        "expires_in": TOKEN_TTL_SECS,
        "expires_at": claims.exp,
        "user": user_view(&user),
    }))
}

/// `POST /api/v1/auth/logout` — stateless JWT, so the client simply drops the
/// token. Answer 200 so UIs can treat it uniformly.
pub async fn logout(_ctx: RequestContext) -> Response {
    Response::json_ok(&json!({
        "status": "ok",
        "detail": "stateless JWT: discard the access token client-side"
    }))
}

/// `GET /api/v1/auth/me` — echo the authenticated user and decoded claims.
pub async fn me(ctx: RequestContext) -> Response {
    let Some(db) = ctx.db.as_ref().map(|d| d.as_ref()) else {
        return internal_error("db_unavailable");
    };

    let user_id = match ctx.get_user_id() {
        Some(id) => id,
        None => return Response::json_unauthorized(&json!({ "error": "unauthorized" })),
    };
    let id: i32 = user_id.parse().unwrap_or_default();

    match system_users::Entity::find_by_id(id).one(db).await {
        Ok(Some(u)) => Response::json_ok(&json!({
            "user": user_view(&u),
            "claims": claims_view(&ctx),
        })),
        Ok(None) => Response::json_not_found(&json!({ "error": "user_not_found", "id": id })),
        Err(e) => internal_error(e),
    }
}
