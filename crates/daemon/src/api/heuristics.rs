//! Heuristic evolver API endpoints.
//!
//! Exposes the [`HeuristicEvolver`] for recording outcomes, triggering
//! evolution, and inspecting the current best heuristic version.

use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::{get, post}};
use serde_json::{json, Value};

use metaygn_core::heuristics::fitness::SessionOutcome;

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /heuristics/outcome -- Record a SessionOutcome for fitness evaluation.
async fn record_outcome(
    State(state): State<AppState>,
    Json(outcome): Json<SessionOutcome>,
) -> Json<Value> {
    let mut evolver = state.evolver.lock().expect("evolver mutex poisoned");
    evolver.record_outcome(outcome);
    Json(json!({ "ok": true }))
}

/// POST /heuristics/evolve -- Trigger one evolution generation, return best.
async fn evolve(
    State(state): State<AppState>,
) -> Json<Value> {
    let mut evolver = state.evolver.lock().expect("evolver mutex poisoned");
    match evolver.evolve_generation() {
        Some(best) => {
            let best_json = serde_json::to_value(best).unwrap_or_default();
            Json(json!({ "ok": true, "best": best_json }))
        }
        None => Json(json!({ "ok": false, "error": "empty population" })),
    }
}

/// GET /heuristics/best -- Return the current best heuristic version.
async fn best(
    State(state): State<AppState>,
) -> Json<Value> {
    let evolver = state.evolver.lock().expect("evolver mutex poisoned");
    match evolver.best() {
        Some(best) => {
            let best_json = serde_json::to_value(best).unwrap_or_default();
            Json(json!({ "best": best_json }))
        }
        None => Json(json!({ "error": "empty population" })),
    }
}

/// GET /heuristics/population -- Return population stats.
async fn population_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let evolver = state.evolver.lock().expect("evolver mutex poisoned");
    let best_fitness = evolver.best().map(|b| b.fitness.composite).unwrap_or(0.0);
    Json(json!({
        "size": evolver.population_size(),
        "generation": evolver.generation(),
        "best_fitness": best_fitness,
    }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/heuristics/outcome", post(record_outcome))
        .route("/heuristics/evolve", post(evolve))
        .route("/heuristics/best", get(best))
        .route("/heuristics/population", get(population_stats))
}
