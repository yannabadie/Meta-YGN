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

#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthDecision {
    Allow,
    WarnAllow,
    Reject(StatusCode),
}

/// Holds the expected bearer token for the running daemon instance.
#[derive(Clone, Debug)]
pub struct AuthToken(pub String);

fn strict_auth_enabled() -> bool {
    std::env::var("METAYGN_STRICT_AUTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn evaluate_auth(path: &str, provided: Option<&str>, expected: &str, strict: bool) -> AuthDecision {
    if path == "/health" {
        return AuthDecision::Allow;
    }

    if provided == Some(expected) {
        return AuthDecision::Allow;
    }

    if strict {
        AuthDecision::Reject(StatusCode::UNAUTHORIZED)
    } else {
        AuthDecision::WarnAllow
    }
}

/// Axum middleware that validates `Authorization: Bearer <token>` on all
/// routes except `/health`.
///
/// During the v2.5 transition period the middleware logs a warning but still
/// allows unauthenticated requests through. Set `METAYGN_STRICT_AUTH=1`
/// to reject them with `401 Unauthorized`.
pub async fn auth_middleware(
    State(token): State<AuthToken>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match evaluate_auth(req.uri().path(), provided, &token.0, strict_auth_enabled()) {
        AuthDecision::Allow => Ok(next.run(req).await),
        AuthDecision::WarnAllow => {
            tracing::warn!(
                path = %req.uri().path(),
                "unauthenticated request allowed (v2.5 compat); set METAYGN_STRICT_AUTH=1 to enforce"
            );
            Ok(next.run(req).await)
        }
        AuthDecision::Reject(status) => {
            tracing::warn!(
                path = %req.uri().path(),
                "rejecting unauthenticated request (strict auth enabled)"
            );
            Err(status)
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

    #[test]
    fn health_route_is_public() {
        assert_eq!(
            evaluate_auth("/health", None, "expected-token", true),
            AuthDecision::Allow
        );
    }

    #[test]
    fn matching_bearer_token_is_allowed() {
        assert_eq!(
            evaluate_auth("/sandbox/exec", Some("expected-token"), "expected-token", true),
            AuthDecision::Allow
        );
    }

    #[test]
    fn compat_mode_allows_missing_token() {
        assert_eq!(
            evaluate_auth("/sandbox/exec", None, "expected-token", false),
            AuthDecision::WarnAllow
        );
    }

    #[test]
    fn strict_mode_rejects_missing_token() {
        assert_eq!(
            evaluate_auth("/sandbox/exec", None, "expected-token", true),
            AuthDecision::Reject(StatusCode::UNAUTHORIZED)
        );
    }

    #[test]
    fn strict_mode_rejects_invalid_token() {
        assert_eq!(
            evaluate_auth("/sandbox/exec", Some("wrong-token"), "expected-token", true),
            AuthDecision::Reject(StatusCode::UNAUTHORIZED)
        );
    }
}
