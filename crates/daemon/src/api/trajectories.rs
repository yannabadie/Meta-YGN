use axum::extract::{Query, State};
use axum::response::Json;
use axum::{Router, routing::get};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct ExportParams {
    limit: Option<u32>,
}

/// GET /trajectories/export?limit=N
///
/// Export recent RL2F trajectories as JSON, ordered by timestamp descending.
async fn export_trajectories(
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> Json<Value> {
    let limit = params.limit.unwrap_or(100);
    match state.memory.export_trajectories(limit).await {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|(id, session_id, trajectory_json, signature_hash, timestamp)| {
                    let trajectory: Value = serde_json::from_str(&trajectory_json)
                        .unwrap_or(Value::String(trajectory_json));
                    json!({
                        "id": id,
                        "session_id": session_id,
                        "trajectory": trajectory,
                        "signature_hash": signature_hash,
                        "timestamp": timestamp,
                    })
                })
                .collect();
            let count = items.len();
            Json(json!({
                "trajectories": items,
                "count": count,
            }))
        }
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/trajectories/export", get(export_trajectories))
}
