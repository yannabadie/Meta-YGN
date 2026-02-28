use axum::extract::State;
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
    let budget = state.budget.lock().expect("budget mutex poisoned");
    let value = serde_json::to_value(&*budget).unwrap_or_default();
    Json(value)
}

/// POST /budget/consume — consume tokens/cost, return updated summary.
async fn consume_budget(
    State(state): State<AppState>,
    Json(req): Json<ConsumeRequest>,
) -> Json<serde_json::Value> {
    let mut budget = state.budget.lock().expect("budget mutex poisoned");
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
        .route("/budget/consume", axum::routing::post(consume_budget))
}
