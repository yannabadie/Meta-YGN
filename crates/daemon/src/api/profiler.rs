use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::{get, post}};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::profiler::fatigue::FatigueReport;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for POST /profiler/signal.
#[derive(Debug, Deserialize)]
pub struct SignalRequest {
    pub signal_type: String,
    /// The prompt text (required when signal_type == "prompt").
    pub prompt: Option<String>,
    /// ISO-8601 timestamp (optional; defaults to now).
    pub timestamp: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /profiler/fatigue -- return current fatigue assessment.
async fn fatigue_report(State(state): State<AppState>) -> Json<FatigueReport> {
    let profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
    Json(profiler.assess())
}

/// POST /profiler/signal -- record a fatigue signal from a hook and return
/// the updated report.
async fn record_signal(
    State(state): State<AppState>,
    Json(req): Json<SignalRequest>,
) -> Json<FatigueReport> {
    let mut profiler = state.fatigue.lock().expect("fatigue mutex poisoned");

    match req.signal_type.as_str() {
        "prompt" => {
            let prompt_text = req.prompt.as_deref().unwrap_or("");
            let ts = req
                .timestamp
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);
            profiler.on_prompt(prompt_text, ts);
        }
        "error" => {
            profiler.on_error();
        }
        "success" => {
            profiler.on_success();
        }
        _ => {
            // Unknown signal type -- ignore gracefully.
        }
    }

    Json(profiler.assess())
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/profiler/fatigue", get(fatigue_report))
        .route("/profiler/signal", post(record_signal))
}
