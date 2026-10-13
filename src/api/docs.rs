use gritshield::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde_json::{json, Value};

use crate::api::{auth, internal_error};
use crate::models::platform_docs;

fn db(ctx: &RequestContext) -> Option<&sea_orm::DatabaseConnection> {
    ctx.db.as_ref().map(|d| d.as_ref())
}

/// `GET /api/v1/docs` — list documents (metadata only). Pass `?slug=` to fetch
/// a single document, markdown body included.
pub async fn list(ctx: RequestContext) -> Response {
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    if let Some(slug) = ctx.query_param("slug") {
        return match platform_docs::Entity::find()
            .filter(platform_docs::Column::Slug.eq(slug))
            .one(db)
            .await
        {
            Ok(Some(doc)) => Response::json_ok(&json!({ "doc": doc })),
            Ok(None) => Response::json_not_found(&json!({ "error": "not_found", "slug": slug })),
            Err(e) => internal_error(e),
        };
    }

    match platform_docs::Entity::find()
        .order_by_asc(platform_docs::Column::Slug)
        .all(db)
        .await
    {
        Ok(rows) => {
            let docs: Vec<Value> = rows
                .iter()
                .map(|d| {
                    json!({
                        "id": d.id,
                        "slug": d.slug,
                        "title": d.title,
                        "created_at": d.created_at,
                    })
                })
                .collect();
            Response::json_ok(&json!({ "count": docs.len(), "docs": docs }))
        }
        Err(e) => internal_error(e),
    }
}

/// `POST /api/v1/docs` — create or upsert a document (Operator+).
/// Body: `{ "slug": "...", "title": "...", "markdown_body": "..." }`.
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

    let slug = body.get("slug").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let markdown_body = body
        .get("markdown_body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if slug.is_empty() || title.is_empty() {
        return Response::json_bad_request(&json!({
            "error": "missing_fields",
            "required": ["slug", "title"]
        }));
    }

    let active = platform_docs::ActiveModel {
        id: NotSet,
        slug: Set(slug),
        title: Set(title),
        markdown_body: Set(markdown_body),
        created_at: NotSet,
    };

    match active.insert(db).await {
        Ok(created) => Response::json_created(&json!({ "doc": created })),
        Err(e) => {
            if e.to_string().to_lowercase().contains("unique") {
                Response::json_conflict(&json!({ "error": "slug_taken" }))
            } else {
                internal_error(e)
            }
        }
    }
}

/// `PUT /api/v1/docs` — update a document by `slug` (Operator+).
pub async fn update(ctx: RequestContext) -> Response {
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

    let slug = match body.get("slug").and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => return Response::json_bad_request(&json!({ "error": "missing_fields", "required": ["slug"] })),
    };

    let existing = match platform_docs::Entity::find()
        .filter(platform_docs::Column::Slug.eq(slug.clone()))
        .one(db)
        .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return Response::json_not_found(&json!({ "error": "not_found", "slug": slug })),
        Err(e) => return internal_error(e),
    };

    let mut active: platform_docs::ActiveModel = existing.into();
    if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
        active.title = Set(title.to_string());
    }
    if let Some(md) = body.get("markdown_body").and_then(|v| v.as_str()) {
        active.markdown_body = Set(md.to_string());
    }

    match active.update(db).await {
        Ok(updated) => Response::json_ok(&json!({ "doc": updated })),
        Err(e) => internal_error(e),
    }
}

/// `DELETE /api/v1/docs?slug=...` — remove a document (Admin).
pub async fn delete(ctx: RequestContext) -> Response {
    if let Some(res) = auth::require_role(&ctx, "Admin") {
        return res;
    }
    let Some(db) = db(&ctx) else {
        return internal_error("db_unavailable");
    };

    let slug = match ctx.query_param("slug") {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return Response::json_bad_request(&json!({ "error": "missing_slug" })),
    };

    match platform_docs::Entity::delete_many()
        .filter(platform_docs::Column::Slug.eq(slug.clone()))
        .exec(db)
        .await
    {
        Ok(res) => Response::json_ok(&json!({ "deleted": res.rows_affected, "slug": slug })),
        Err(e) => internal_error(e),
    }
}
