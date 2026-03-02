use axum::response::Json;
use axum::{Router, routing::get};
use serde_json::{Value, json};

use crate::app_state::AppState;

/// GET /health
async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "kernel": "verified"
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
