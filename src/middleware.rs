use async_trait::async_trait;
use gritshield::http::response::Response;
use gritshield::middleware::{Middleware, MiddlewareResult};
use gritshield::prelude::RequestContext;

/// Attaches hardening headers to every response and exposes the request
/// duration, mirroring the sibling CyberFlow/LuxShield backends.
pub struct SecurityHeaders;

impl SecurityHeaders {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SecurityHeaders {
    async fn on_request(&self, _ctx: &mut RequestContext) -> MiddlewareResult {
        MiddlewareResult::Next(None)
    }

    async fn on_response(&self, ctx: &RequestContext, res: &mut Response) {
        push_header(res, "X-Content-Type-Options", "nosniff");
        push_header(res, "X-Frame-Options", "DENY");
        push_header(res, "Referrer-Policy", "no-referrer");
        push_header(res, "Cache-Control", "no-store");
        push_header(res, "Server", "admin-command-center");
        res.headers.push((
            "X-ACC-Response-Time-Ms".to_string(),
            ctx.start_time.elapsed().as_millis().to_string(),
        ));
    }
}

fn push_header(res: &mut Response, key: &str, value: impl Into<String>) {
    if !res.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(key)) {
        res.headers.push((key.to_string(), value.into()));
    }
}
