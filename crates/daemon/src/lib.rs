pub mod api;
pub mod app_state;
pub mod auth;
pub mod forge;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod metrics;
pub mod postprocess;
pub mod profiler;
pub mod proxy;
#[cfg(feature = "semantic")]
pub mod semantic_router;
pub mod session;
pub mod verification;

use anyhow::Result;
use axum::Router;
use tokio::sync::watch;

use crate::app_state::AppState;
use crate::auth::AuthToken;

/// Build the app with an in-memory database (for tests).
/// Includes a dummy shutdown sender so `/admin/shutdown` works in tests.
/// Uses a fixed test token for the auth middleware.
pub async fn build_app() -> Result<Router> {
    let state = AppState::new_in_memory().await?;
    let (shutdown_tx, _shutdown_rx) = watch::channel(false);
    let token = AuthToken("test-token".to_string());
    Ok(build_app_with_state(state, token).layer(axum::Extension(shutdown_tx)))
}

/// Build the app with the given state and auth token (shared builder).
/// Callers are responsible for layering `Extension<watch::Sender<bool>>`
/// for the `/admin/shutdown` endpoint.
pub fn build_app_with_state(state: AppState, token: AuthToken) -> Router {
    api::router(state).layer(axum::middleware::from_fn_with_state(
        token,
        auth::auth_middleware,
    ))
}
