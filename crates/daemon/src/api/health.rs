use axum::{Router, routing::get};
use axum::response::Json;
use serde_json::{json, Value};

use crate::app_state::AppState;

/// GET /health
async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": "0.1.0",
        "kernel": "verified"
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
