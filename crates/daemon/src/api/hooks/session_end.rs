use axum::extract::State;
use axum::response::Json;

use metaygn_core::context::LoopContext;
use metaygn_shared::protocol::{HookInput, HookOutput};

use crate::app_state::AppState;

/// POST /hooks/session-end
///
/// Fire-and-forget endpoint called by the SessionEnd hook.
/// Logs the session closure event for replay and returns immediately.
/// Note: session cleanup is already performed by the Stop handler;
/// this endpoint ensures the daemon has a record of session termination.
pub(crate) async fn session_end(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event(&session_id, "session_end", &payload)
        .await;

    // Ensure session context is cleaned up (idempotent if Stop already ran)
    state.sessions.remove(&session_id);

    HookOutput::context("SessionEnd".to_string(), "Session ended.".to_string()).into()
}

/// POST /hooks/analyze
///
/// Debug endpoint: runs the full 12-stage loop on an input and returns the
/// complete LoopContext as JSON for inspection.
pub(crate) async fn analyze(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<serde_json::Value> {
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("analyze", "analyze", &payload).await;

    let mut ctx = LoopContext::new(input);
    state.control_loop.run(&mut ctx);

    // Return the full context as JSON
    let value = serde_json::to_value(&ctx).unwrap_or_default();
    Json(value)
}
