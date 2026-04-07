use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};
use serde::Deserialize;
use std::time::Duration;

use metaygn_sandbox::{Hypothesis, ProcessSandbox, SandboxConfig, SandboxResult};

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for POST /sandbox/exec.
#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    pub language: String,
    pub code: String,
    /// Optional timeout override for this request.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Body for POST /sandbox/hypothesis.
#[derive(Debug, Deserialize)]
pub struct HypothesisRequest {
    pub description: String,
    pub language: String,
    pub code: String,
    #[serde(default = "default_expected_success")]
    pub expected_success: bool,
}

fn default_expected_success() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /sandbox/exec -- execute code in the process sandbox.
async fn exec(State(state): State<AppState>, Json(req): Json<ExecRequest>) -> Json<SandboxResult> {
    let result = if let Some(timeout_ms) = req.timeout_ms {
        let timeout_sandbox = ProcessSandbox::new(SandboxConfig {
            timeout: Duration::from_millis(timeout_ms.max(1)),
            ..SandboxConfig::default()
        });
        timeout_sandbox.execute(&req.language, &req.code).await
    } else {
        state.sandbox.execute(&req.language, &req.code).await
    };

    let result = match result {
        Ok(r) => r,
        Err(e) => SandboxResult {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: e.to_string(),
            duration_ms: 0,
            timed_out: false,
        },
    };
    Json(result)
}

/// POST /sandbox/hypothesis -- test a hypothesis in the sandbox.
async fn hypothesis(
    State(state): State<AppState>,
    Json(req): Json<HypothesisRequest>,
) -> Json<SandboxResult> {
    let h = Hypothesis {
        description: req.description,
        language: req.language,
        code: req.code,
        expected_success: req.expected_success,
    };
    let result = state.sandbox.test_hypothesis(&h).await;
    Json(result)
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sandbox/exec", post(exec))
        .route("/sandbox/hypothesis", post(hypothesis))
}
