use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};

use metaygn_core::context::LoopContext;
use metaygn_shared::protocol::{HookInput, HookOutput, PermissionDecision};
use metaygn_shared::state::RiskLevel;

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the command string from tool_input. The command may be in
/// tool_input.command (Bash tool) or tool_input itself might be a string.
fn extract_command(input: &HookInput) -> String {
    if let Some(ref tool_input) = input.tool_input {
        // Try tool_input.command (Bash tool pattern)
        if let Some(cmd) = tool_input.get("command").and_then(|v| v.as_str()) {
            return cmd.to_string();
        }
        // Try tool_input.input (Write/Edit tool pattern)
        if let Some(cmd) = tool_input.get("input").and_then(|v| v.as_str()) {
            return cmd.to_string();
        }
        // Fallback: serialize the entire tool_input
        return tool_input.to_string();
    }
    String::new()
}

/// Determine whether a tool response indicates an error.
fn response_looks_like_error(tool_name: &str, response: &str) -> bool {
    if tool_name == "Bash" && (response.contains("FAIL") || response.contains("error")) {
        return true;
    }
    if response.starts_with("Error") || response.starts_with("error:") {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// POST /hooks/pre-tool-use
///
/// 1. Run the GuardPipeline on tool_name + command
/// 2. If the pipeline blocks -> return deny (score 0) or ask (score > 0)
/// 3. Run ControlLoop stages 1-6 (classify through strategy) for risk assessment
/// 4. Return enriched context with risk level and strategy recommendation
async fn pre_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let command = extract_command(&input);

    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "pre_tool_use", &payload).await;

    // Step 1: Run the guard pipeline
    let pipeline_decision = state.guard_pipeline.check(&tool_name, &command);

    if !pipeline_decision.allowed {
        // Score 0 = destructive -> deny; score > 0 = high-risk -> ask
        let decision = if pipeline_decision.aggregate_score == 0 {
            PermissionDecision::Deny
        } else {
            PermissionDecision::Ask
        };

        let reason = pipeline_decision
            .blocking_guard
            .as_deref()
            .map(|g| {
                let detail = pipeline_decision.results.iter()
                    .find(|r| r.guard_name == g)
                    .and_then(|r| r.reason.as_deref())
                    .unwrap_or("blocked by guard");
                format!("[guard:{g}] {detail}")
            })
            .unwrap_or_else(|| "Blocked by guard pipeline".to_string());

        return Json(HookOutput::permission(decision, reason));
    }

    // Step 2: Run ControlLoop stages 1-6 for risk assessment
    let mut ctx = LoopContext::new(input);
    state.control_loop.run_range(&mut ctx, 0, 6);

    let risk_label = match ctx.risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    };

    // Return enriched context with risk + strategy
    Json(HookOutput::context(
        "PreToolUse".to_string(),
        format!(
            "[risk:{}] [strategy:{:?}] [difficulty:{:.2}] Tool '{}' allowed by guard pipeline (score:{})",
            risk_label, ctx.strategy, ctx.difficulty, tool_name, pipeline_decision.aggregate_score,
        ),
    ))
}

/// POST /hooks/post-tool-use
async fn post_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    // Log the tool output for verification signals
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "post_tool_use", &payload).await;

    // Post-tool-use: emit verification context based on what happened
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let response = input.tool_response.clone().unwrap_or_default();

    // Wire fatigue signals: record error or success
    {
        let mut profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
        if response_looks_like_error(&tool_name, &response) {
            profiler.on_error();
        } else {
            profiler.on_success();
        }
    }

    let context = if tool_name == "Bash" && response.contains("FAIL") {
        "Test failure detected in Bash output. Review results before proceeding."
    } else if tool_name == "Write" || tool_name == "Edit" {
        "File modification recorded. Verify changes align with intent."
    } else if tool_name.starts_with("mcp__") {
        "MCP tool output recorded. Verify external system state."
    } else {
        "Tool output recorded."
    };

    Json(HookOutput::context(
        "PostToolUse".to_string(),
        context.to_string(),
    ))
}

/// POST /hooks/user-prompt-submit
///
/// 1. Run ControlLoop stages 1-6 on the user's prompt
/// 2. Record a fatigue signal for the prompt
/// 3. Return risk, strategy, and budget recommendation
async fn user_prompt_submit(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "user_prompt_submit", &payload).await;

    // Wire fatigue signal: record the prompt
    {
        let prompt_text = input.prompt.as_deref().unwrap_or("");
        let mut profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
        profiler.on_prompt(prompt_text, chrono::Utc::now());
    }

    // Run ControlLoop stages 1-6 for analysis
    let mut ctx = LoopContext::new(input);
    state.control_loop.run_range(&mut ctx, 0, 6);

    let risk_label = match ctx.risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    };

    Json(HookOutput::context(
        "UserPromptSubmit".to_string(),
        format!(
            "[risk:{}] [strategy:{:?}] [budget:{}tok] [task:{:?}] Prompt analysed via control loop",
            risk_label, ctx.strategy, ctx.budget.max_tokens, ctx.task_type,
        ),
    ))
}

/// POST /hooks/stop
///
/// Run ControlLoop stages 9-12 (calibrate, compact, decide, learn)
/// and return a proof-packet enforcement hint.
async fn stop(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "stop", &payload).await;

    let mut ctx = LoopContext::new(input);
    let decision = state.control_loop.run_range(&mut ctx, 8, 12);

    let metacog = ctx.metacog_vector.compact_encode();
    let lessons_summary = if ctx.lessons.is_empty() {
        "none".to_string()
    } else {
        ctx.lessons.join("; ")
    };

    Json(HookOutput::context(
        "Stop".to_string(),
        format!(
            "[decision:{:?}] [metacog:{}] [lessons:{}] Proof packet enforcement recommended",
            decision, metacog, lessons_summary,
        ),
    ))
}

/// POST /hooks/analyze
///
/// Debug endpoint: runs the full 12-stage loop on an input and returns the
/// complete LoopContext as JSON for inspection.
async fn analyze(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<serde_json::Value> {
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "analyze", &payload).await;

    let mut ctx = LoopContext::new(input);
    state.control_loop.run(&mut ctx);

    // Return the full context as JSON
    let value = serde_json::to_value(&ctx).unwrap_or_default();
    Json(value)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hooks/pre-tool-use", post(pre_tool_use))
        .route("/hooks/post-tool-use", post(post_tool_use))
        .route("/hooks/user-prompt-submit", post(user_prompt_submit))
        .route("/hooks/stop", post(stop))
        .route("/hooks/analyze", post(analyze))
}
