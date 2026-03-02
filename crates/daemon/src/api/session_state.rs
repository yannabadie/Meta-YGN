use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::{Router, routing::get};

use crate::app_state::AppState;

/// GET /session/{session_id}/state
///
/// Returns the full structured session state as JSON.
/// Unlike hook responses (which return text in `additionalContext`),
/// this endpoint exposes the daemon's accumulated SessionContext
/// for structured querying by Claude or external tools.
async fn get_session_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.sessions.get(&session_id) {
        Some(ctx) => {
            let sess = ctx.lock().unwrap();
            (StatusCode::OK, Json(serde_json::json!({
                "session_id": sess.session_id,
                "task_type": sess.task_type.map(|t| format!("{:?}", t)),
                "risk": format!("{:?}", sess.risk),
                "strategy": format!("{:?}", sess.strategy),
                "difficulty": sess.difficulty,
                "competence": sess.competence,
                "tool_calls": sess.tool_calls,
                "errors": sess.errors,
                "success_count": sess.success_count,
                "tokens_consumed": sess.tokens_consumed,
                "fatigue_score": sess.fatigue.assess().score,
                "lessons": &sess.lessons,
                "verification_results": &sess.verification_results,
                "has_execution_plan": sess.execution_plan.is_some(),
            })))
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "session not found"}))),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/session/{session_id}/state", get(get_session_state))
}
