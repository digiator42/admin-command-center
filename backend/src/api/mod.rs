pub mod auth;
pub mod control;
pub mod docs;
pub mod health;
pub mod integrations;
pub mod services;
pub mod system;
pub mod users;

use gritshield::prelude::Response;
use serde_json::json;

/// Uniform 500 helper so every handler reports DB/serialization failures the
/// same way.
pub fn internal_error(msg: impl std::fmt::Display) -> Response {
    Response::json_internal_error(&json!({ "error": msg.to_string() }))
}
