pub mod api;
pub mod app_state;
pub mod forge;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod postprocess;
pub mod profiler;
pub mod proxy;
pub mod session;
pub mod verification;

use anyhow::Result;
use axum::Router;
use tokio::sync::watch;

use crate::app_state::AppState;

/// Build the app with an in-memory database (for tests).
/// Includes a dummy shutdown sender so `/admin/shutdown` works in tests.
pub async fn build_app() -> Result<Router> {
    let state = AppState::new_in_memory().await?;
    let (shutdown_tx, _shutdown_rx) = watch::channel(false);
    Ok(build_app_with_state(state).layer(axum::Extension(shutdown_tx)))
}

/// Build the app with the given state (shared builder).
/// Callers are responsible for layering `Extension<watch::Sender<bool>>`
/// for the `/admin/shutdown` endpoint.
pub fn build_app_with_state(state: AppState) -> Router {
    api::router(state)
}
