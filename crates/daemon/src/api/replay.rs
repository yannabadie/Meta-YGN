use axum::extract::{Path, State};
use axum::response::Json;
use axum::{Router, routing::get};
use serde_json::{Value, json};

use crate::app_state::AppState;

/// GET /replay/sessions
///
/// List all recorded sessions with event counts and time range.
async fn list_sessions(State(state): State<AppState>) -> Json<Value> {
    match state.memory.replay_sessions().await {
        Ok(sessions) => {
            let items: Vec<Value> = sessions
                .into_iter()
                .map(|(session_id, event_count, first_event, last_event)| {
                    json!({
                        "session_id": session_id,
                        "event_count": event_count,
                        "first_event": first_event,
                        "last_event": last_event,
                    })
                })
                .collect();
            Json(json!({ "sessions": items }))
        }
        Err(e) => Json(json!({
            "error": format!("Failed to list replay sessions: {}", e),
            "sessions": []
        })),
    }
}

/// GET /replay/{session_id}
///
/// Retrieve all replay events for a given session, with request/response
/// parsed back to JSON values.
async fn get_session(State(state): State<AppState>, Path(session_id): Path<String>) -> Json<Value> {
    match state.memory.replay_events(&session_id).await {
        Ok(events) => {
            let items: Vec<Value> = events
                .into_iter()
                .map(
                    |(id, hook_event, request_json, response_json, latency_ms, timestamp)| {
                        let request: Value = serde_json::from_str(&request_json)
                            .unwrap_or(Value::String(request_json));
                        let response: Value = serde_json::from_str(&response_json)
                            .unwrap_or(Value::String(response_json));
                        json!({
                            "id": id,
                            "hook_event": hook_event,
                            "request": request,
                            "response": response,
                            "latency_ms": latency_ms,
                            "timestamp": timestamp,
                        })
                    },
                )
                .collect();
            Json(json!({
                "session_id": session_id,
                "events": items,
            }))
        }
        Err(e) => Json(json!({
            "error": format!("Failed to retrieve replay events: {}", e),
            "session_id": session_id,
            "events": []
        })),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/replay/sessions", get(list_sessions))
        .route("/replay/{session_id}", get(get_session))
}
