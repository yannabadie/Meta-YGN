pub mod health;
pub mod hooks;
pub mod memory;
pub mod profiler;
pub mod sandbox;

use axum::Router;

use crate::app_state::AppState;

/// Build the full router with all routes.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(hooks::routes())
        .merge(memory::routes())
        .merge(sandbox::routes())
        .merge(profiler::routes())
        .with_state(state)
}
