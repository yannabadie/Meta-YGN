pub mod api;
pub mod app_state;

use anyhow::Result;
use axum::Router;

use crate::app_state::AppState;

/// Build the app with an in-memory database (for tests).
pub async fn build_app() -> Result<Router> {
    let state = AppState::new_in_memory().await?;
    Ok(build_app_with_state(state))
}

/// Build the app with the given state (shared builder).
pub fn build_app_with_state(state: AppState) -> Router {
    api::router(state)
}
