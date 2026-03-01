//! Forge API endpoints.
//!
//! Exposes the [`ForgeEngine`] for generating tools from templates,
//! executing tool specs, and listing available templates.

use std::collections::HashMap;

use axum::extract::State;
use axum::response::Json;
use axum::{
    Router,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app_state::AppState;
use crate::forge::{ToolSpec, list_templates};

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

/// Body for `POST /forge/generate`.
#[derive(Deserialize)]
pub struct GenerateRequest {
    pub template: String,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

/// Body for `POST /forge/execute`.
#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub spec: ToolSpec,
    #[serde(default)]
    pub input: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /forge/generate -- Generate a tool from a named template.
async fn generate(State(state): State<AppState>, Json(req): Json<GenerateRequest>) -> Json<Value> {
    let mut forge = state.forge.lock().expect("forge mutex poisoned");
    match forge.generate(&req.template, &req.params) {
        Ok(spec) => {
            let spec_json = serde_json::to_value(&spec).unwrap_or_default();
            Json(json!({ "ok": true, "spec": spec_json }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// POST /forge/execute -- Execute a ToolSpec with given input.
///
/// Note: We clone the sandbox Arc from the forge so we can drop the mutex
/// before the async execution.
async fn execute(State(state): State<AppState>, Json(req): Json<ExecuteRequest>) -> Json<Value> {
    // We cannot hold the Mutex across an await, so use the sandbox directly
    // via the AppState (which also holds an Arc<ProcessSandbox>).
    let forge = state.forge.lock().expect("forge mutex poisoned");
    // ForgeEngine::execute only needs &self (no mutation), but we hold the
    // lock for the entire call.  To avoid holding across await, build a
    // temporary engine with the same sandbox.
    drop(forge);

    // Build a temporary ForgeEngine that shares the same sandbox.
    let tmp_forge = crate::forge::ForgeEngine::new(state.sandbox.clone());
    match tmp_forge.execute(&req.spec, &req.input).await {
        Ok(result) => {
            let result_json = serde_json::to_value(&result).unwrap_or_default();
            Json(json!({ "ok": true, "result": result_json }))
        }
        Err(e) => Json(json!({ "ok": false, "error": e.to_string() })),
    }
}

/// GET /forge/templates -- List available template names.
async fn templates() -> Json<Value> {
    let names = list_templates();
    let items: Vec<Value> = names
        .into_iter()
        .map(|name| {
            let tmpl = crate::forge::get_template(name).unwrap();
            json!({
                "name": tmpl.name,
                "description": tmpl.description,
                "language": format!("{:?}", tmpl.language),
                "params": tmpl.params,
            })
        })
        .collect();
    Json(json!({ "templates": items }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/forge/generate", post(generate))
        .route("/forge/execute", post(execute))
        .route("/forge/templates", get(templates))
}
