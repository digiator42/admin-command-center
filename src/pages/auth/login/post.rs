use gritshield::prelude::*;
use gritshield::protocol::request::HttpMethod;
use gritshield::register_page;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::system_users;
use crate::security::crypto::{generate_hash, verify_password};

// Handle Authentication Attempt (POST /login)
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

    // Look up user row matching input username
    if let Ok(Some(user)) = system_users::Entity::find()
        .filter(system_users::Column::Username.eq(username))
        .one(db.as_ref())
        .await
    {
        // Crypto confirmation sequence matches hash validation
        if verify_password(password.trim(), &user.password_hash.trim()) {
            // Update the session in memory. Since it's thread-safe and tracked by GritShield's
            // global SessionStore pool, this change is instantly live across all threads!
            if let Some(ref session_arc) = ctx.session {
                let mut session = session_arc.lock().unwrap();
                session
                    .data
                    .insert("user_id".to_string(), user.id.to_string());
                session
                    .data
                    .insert("role".to_string(), user.role.to_string()); // "SuperAdmin"

                println!(
                    "[SUCCESS] Session updated globally in store: {:?}",
                    session.data
                );
            }

            // Direct clean browser redirect
            return Response::redirect(303, "/dashboard");
        }
    }

    // Edge-case identity authentication denial
    return Response::new(401, Sanitizer::trust(
        "<h1>401 Unauthorized</h1><p>No user found with the provided credentials.</p>".into()
    ));
}

register_page!(HttpMethod::POST, post_handler);
