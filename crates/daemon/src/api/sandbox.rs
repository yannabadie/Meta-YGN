use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};
use serde::Deserialize;
use std::time::Duration;

use metaygn_sandbox::{Hypothesis, ProcessSandbox, SandboxConfig, SandboxResult};

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// WASM endpoint (feature-gated)
// ---------------------------------------------------------------------------

/// Body for POST /sandbox/wasm.
#[cfg(feature = "wasm")]
#[derive(serde::Deserialize)]
pub struct WasmExecRequest {
    pub wat: String,
    pub timeout_ms: Option<u64>,
}

#[cfg(feature = "wasm")]
async fn exec_wasm(Json(req): Json<WasmExecRequest>) -> Json<SandboxResult> {
    use metaygn_sandbox::wasm_sandbox::{WasmSandbox, WasmSandboxConfig};

    let config = WasmSandboxConfig {
        timeout_ms: req.timeout_ms.unwrap_or(5000).clamp(1, 30_000), // max 30 s
        ..Default::default()
    };

    let sandbox = match WasmSandbox::new(config) {
        Ok(s) => s,
        Err(e) => {
            return Json(SandboxResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("Failed to create WASM sandbox: {e}"),
                duration_ms: 0,
                timed_out: false,
            })
        }
    };

    Json(sandbox.execute_wat(&req.wat))
}

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
    // TODO(security): Add bearer-token authentication. The daemon binds to
    // 127.0.0.1 only, but any local process can execute arbitrary code via
    // this endpoint. See security audit finding S5.
    let result = if let Some(timeout_ms) = req.timeout_ms {
        let timeout_sandbox = ProcessSandbox::new(SandboxConfig {
            timeout: Duration::from_millis(timeout_ms.clamp(1, 30_000)), // max 30 s
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
    let router = Router::new()
        .route("/sandbox/exec", post(exec))
        .route("/sandbox/hypothesis", post(hypothesis));

    #[cfg(feature = "wasm")]
    let router = router.route("/sandbox/wasm", post(exec_wasm));

    router
}
