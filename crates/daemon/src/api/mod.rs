pub mod admin;
pub mod budget;
pub mod forge;
pub mod graph;
pub mod health;
pub mod heuristics;
pub mod hooks;
pub mod memory;
pub mod profiler;
pub mod replay;
pub mod sandbox;

use axum::routing::post;
use axum::Router;

use crate::app_state::AppState;
use crate::proxy::service;

/// Build the full router with all routes.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes())
        .merge(hooks::routes())
        .merge(memory::routes())
        .merge(sandbox::routes())
        .merge(profiler::routes())
        .merge(graph::routes())
        .merge(heuristics::routes())
        .merge(forge::routes())
        .merge(budget::routes())
        .merge(replay::routes())
        .merge(admin::routes())
        .route("/proxy/anthropic", post(service::prune_messages))
        .with_state(state)
}
