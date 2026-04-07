use axum::extract::{Path, State};
use axum::response::Json;
use axum::{Router, routing::get};
use serde::Deserialize;

use crate::app_state::AppState;

/// Request body for POST /budget/consume.
#[derive(Debug, Deserialize)]
pub struct ConsumeRequest {
    pub tokens: u64,
    pub cost_usd: f64,
}

/// GET /budget — returns the full budget status as JSON.
async fn get_budget(State(state): State<AppState>) -> Json<serde_json::Value> {
    let Ok(budget) = state.budget.lock() else {
        tracing::warn!("budget mutex poisoned");
        return Json(serde_json::json!({"error": "budget mutex poisoned"}));
    };
    let value = serde_json::to_value(&*budget).unwrap_or_default();
    Json(value)
}

/// GET /budget/:session_id — returns session-local budget.
async fn get_session_budget(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Json<serde_json::Value> {
    let sess_arc = state.sessions.get_or_create(&session_id);
    let Ok(sess) = sess_arc.lock() else {
        tracing::warn!("session mutex poisoned");
        return Json(serde_json::json!({"error": "session mutex poisoned"}));
    };
    let value = serde_json::to_value(&sess.budget).unwrap_or_default();
    Json(value)
}

/// POST /budget/consume — consume tokens/cost, return updated summary.
async fn consume_budget(
    State(state): State<AppState>,
    Json(req): Json<ConsumeRequest>,
) -> Json<serde_json::Value> {
    let Ok(mut budget) = state.budget.lock() else {
        tracing::warn!("budget mutex poisoned");
        return Json(serde_json::json!({"error": "budget mutex poisoned"}));
    };
    budget.consume(req.tokens, req.cost_usd);
    let summary = budget.summary();
    let over = budget.is_over_budget();
    let warn = budget.should_warn();
    Json(serde_json::json!({
        "summary": summary,
        "is_over_budget": over,
        "should_warn": warn,
        "consumed_tokens": budget.consumed_tokens(),
        "consumed_cost_usd": budget.consumed_cost_usd(),
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/budget", get(get_budget))
        .route("/budget/{session_id}", get(get_session_budget))
        .route("/budget/consume", axum::routing::post(consume_budget))
}
