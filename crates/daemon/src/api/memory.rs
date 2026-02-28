use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::{get, post}};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::app_state::AppState;

/// Request body for POST /memory/recall.
#[derive(Deserialize)]
pub struct RecallRequest {
    pub query: String,
    pub limit: Option<u32>,
}

/// POST /memory/recall — FTS search over events.
async fn recall(
    State(state): State<AppState>,
    Json(req): Json<RecallRequest>,
) -> Json<Value> {
    let limit = req.limit.unwrap_or(10);
    match state.memory.search_events(&req.query, limit).await {
        Ok(rows) => {
            let events: Vec<Value> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "session_id": r.session_id,
                        "event_type": r.event_type,
                        "payload": r.payload,
                        "timestamp": r.timestamp,
                    })
                })
                .collect();
            Json(json!({ "events": events }))
        }
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// GET /memory/stats — Return event count.
async fn stats(State(state): State<AppState>) -> Json<Value> {
    match state.memory.event_count().await {
        Ok(count) => Json(json!({ "event_count": count })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/memory/recall", post(recall))
        .route("/memory/stats", get(stats))
}
