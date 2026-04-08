//! Bearer-token authentication middleware for the daemon HTTP server.
//!
//! At startup the daemon generates a random token, writes it to
//! `~/.claude/aletheia/daemon.token`, and installs this middleware on every
//! route **except** `/health`.
//!
//! In v2.5 backward-compatibility mode the middleware only *warns* on a
//! missing or invalid token instead of rejecting the request outright.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Holds the expected bearer token for the running daemon instance.
#[derive(Clone, Debug)]
pub struct AuthToken(pub String);

/// Axum middleware that validates `Authorization: Bearer <token>` on all
/// routes except `/health`.
///
/// During the v2.5 transition period the middleware logs a warning but still
/// allows unauthenticated requests through.  Set `METAYGN_STRICT_AUTH=1`
/// to reject them with `401 Unauthorized`.
pub async fn auth_middleware(
    State(token): State<AuthToken>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // /health is always public so monitoring tools work without a token.
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(t) if t == token.0 => Ok(next.run(req).await),
        _ => {
            // v2.5 backward compatibility: warn but allow through.
            // Flip to strict rejection via env var for early adopters.
            let strict = std::env::var("METAYGN_STRICT_AUTH")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);

            if strict {
                tracing::warn!(
                    path = %req.uri().path(),
                    "rejecting unauthenticated request (strict auth enabled)"
                );
                Err(StatusCode::UNAUTHORIZED)
            } else {
                tracing::warn!(
                    path = %req.uri().path(),
                    "unauthenticated request allowed (v2.5 compat) — \
                     set METAYGN_STRICT_AUTH=1 to enforce"
                );
                Ok(next.run(req).await)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_token_is_cloneable() {
        let t = AuthToken("test-token".to_string());
        let t2 = t.clone();
        assert_eq!(t.0, t2.0);
    }
}
