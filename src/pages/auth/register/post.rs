use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::models::system_users;
use crate::security::crypto::generate_hash;

// Process submission parameters (POST /register)
// Process submission parameters (POST /register)
pub async fn post_handler(ctx: RequestContext) -> Response {
    let db = match ctx.db {
        Some(ref pool) => pool,
        None => {
            return Response::new(
                500,
                Sanitizer::trust("Database context pool missing".into()),
            );
        }
    };

    let form_data = ctx.req.parse_form_body();

    // Fix: Call .trim() immediately on your input parameters during registration
    let username = form_data
        .fields
        .get("username")
        .map(|s| s.as_str().trim())
        .unwrap_or("");
    let password = form_data
        .fields
        .get("password")
        .map(|s| s.as_str().trim())
        .unwrap_or("");
    let role = form_data
        .fields
        .get("role")
        .map(|s| s.as_str().trim())
        .unwrap_or("Auditor");

    if username.is_empty() || password.is_empty() {
        return Response::new(
            400,
            Sanitizer::trust("Missing required creation credentials".into()),
        );
    }

    // Now this hash is generated from a perfectly clean plain-text string!
    let hashed = match generate_hash(password) {
        Ok(h) => h,
        Err(_) => return Response::new(500, Sanitizer::trust("Internal crypto fault".into())),
    };

    let new_user = system_users::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        username: Set(username.to_string()),
        password_hash: Set(hashed),
        role: Set(role.to_string()),
        created_at: Set(None),
    };

    match new_user.insert(db.as_ref()).await {
        Ok(_) => {
            let mut res = Response::new(303, Sanitizer::trust("Redirecting...".into()));
            res.headers
                .push(("Location".to_string(), "/login".to_string()));
            res
        }
        Err(err) => Response::new(
            400,
            Sanitizer::trust(
                format!(
                    "Account deployment rejected: User might already exist ({})",
                    err
                )
                .as_str(),
            ),
        ),
    }
}

register_page!(HttpMethod::POST, post_handler);
