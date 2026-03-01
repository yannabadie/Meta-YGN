use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};

use metaygn_core::context::LoopContext;
use metaygn_core::topology::TopologyPlanner;
use metaygn_shared::protocol::{HookInput, HookOutput, PermissionDecision};
use metaygn_shared::state::{RiskLevel, TaskType};

use crate::app_state::AppState;
use crate::proxy::pruner::ContextPruner;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Append the session budget summary to a HookOutput's additionalContext.
fn append_budget_to_output(output: &mut HookOutput, state: &AppState) {
    let budget = state.budget.lock().expect("budget mutex poisoned");
    let summary = budget.summary();
    match output.hook_specific_output {
        Some(ref mut hso) => {
            // Append to existing additionalContext, or create it
            match hso.additional_context {
                Some(ref mut ctx) => {
                    ctx.push(' ');
                    ctx.push_str(&summary);
                }
                None => {
                    hso.additional_context = Some(summary);
                }
            }
        }
        None => {
            output.hook_specific_output = Some(metaygn_shared::protocol::HookSpecificOutput {
                additional_context: Some(summary),
                ..Default::default()
            });
        }
    }
}

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

/// Format a human-readable context line from risk, strategy, budget, task, and topology.
fn format_readable(
    risk: &str,
    strategy: &str,
    budget_tokens: u64,
    task: &str,
    topology: &str,
) -> String {
    format!(
        "Risk: {} | Strategy: {} | Budget: {} tokens | Task: {} | Topology: {}",
        risk.to_uppercase(),
        strategy,
        budget_tokens,
        task,
        topology,
    )
}

/// Append a latency tag to the additionalContext field of a HookOutput.
fn append_latency(output: &mut HookOutput, start: std::time::Instant) {
    let latency_ms = start.elapsed().as_millis();
    let tag = format!(" [latency: {}ms]", latency_ms);
    if let Some(ref mut hso) = output.hook_specific_output {
        if let Some(ref mut ctx) = hso.additional_context {
            ctx.push_str(&tag);
        } else {
            hso.additional_context = Some(tag);
        }
    }
}

/// Record a replay event for session replay, ignoring errors.
async fn record_replay(
    state: &AppState,
    session_id: &str,
    hook_event: &str,
    request_json: &str,
    response_json: &str,
    start: std::time::Instant,
) {
    let latency_ms = start.elapsed().as_millis() as i64;
    let _ = state
        .memory
        .record_replay_event(
            session_id,
            hook_event,
            request_json,
            response_json,
            latency_ms,
        )
        .await;
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
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let command = extract_command(&input);

    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event("daemon", "pre_tool_use", &payload)
        .await;

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
                let detail = pipeline_decision
                    .results
                    .iter()
                    .find(|r| r.guard_name == g)
                    .and_then(|r| r.reason.as_deref())
                    .unwrap_or("blocked by guard");
                format!("[guard:{g}] {detail}")
            })
            .unwrap_or_else(|| "Blocked by guard pipeline".to_string());

        let mut output = HookOutput::permission(decision, reason);
        append_budget_to_output(&mut output, &state);
        append_latency(&mut output, start);
        let resp_json = serde_json::to_string(&output).unwrap_or_default();
        record_replay(
            &state,
            &session_id,
            "PreToolUse",
            &payload,
            &resp_json,
            start,
        )
        .await;
        return Json(output);
    }

    // Test Integrity Guard: detect test modification instead of code fixing
    if (tool_name == "Edit" || tool_name == "MultiEdit")
        && let Some(ref tool_input) = input.tool_input
        && let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str())
    {
        let old_string = tool_input
            .get("old_string")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let new_string = tool_input
            .get("new_string")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let report =
            metaygn_verifiers::test_integrity::analyze_test_edit(file_path, old_string, new_string);

        if report.suspicious {
            let issues_detail = report
                .issues
                .iter()
                .map(|i| i.detail.as_str())
                .collect::<Vec<_>>()
                .join("; ");

            let mut output = HookOutput {
                hook_specific_output: Some(metaygn_shared::protocol::HookSpecificOutput {
                    hook_event_name: Some("PreToolUse".into()),
                    permission_decision: Some(PermissionDecision::Ask),
                    permission_decision_reason: Some(report.recommendation),
                    additional_context: Some(format!("Issues: {}", issues_detail)),
                }),
            };
            append_budget_to_output(&mut output, &state);
            append_latency(&mut output, start);
            let resp_json = serde_json::to_string(&output).unwrap_or_default();
            record_replay(
                &state,
                &session_id,
                "PreToolUse",
                &payload,
                &resp_json,
                start,
            )
            .await;
            return Json(output);
        }
    }

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // Step 2: Run ControlLoop stages 1-6 for risk assessment
    // Fix: inject the actual command text as the prompt so the classify/assess
    // stages can evaluate the real content rather than defaulting to high risk
    // based solely on the tool name "Bash".
    let mut input_for_loop = input.clone();
    let cmd = extract_command(&input_for_loop);
    if !cmd.is_empty() {
        input_for_loop.prompt = Some(cmd);
    }
    let mut ctx = LoopContext::new(input_for_loop);
    state.control_loop.run_range(&mut ctx, 0, 6);

    // Persist tool call count into session context
    {
        let mut sess = session_ctx.lock().unwrap();
        sess.tool_calls += 1;
    }

    let risk_label = match ctx.risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    };

    // Return enriched context with risk + strategy
    let mut output = HookOutput::context(
        "PreToolUse".to_string(),
        format!(
            "[risk:{}] [strategy:{:?}] [difficulty:{:.2}] Tool '{}' allowed by guard pipeline (score:{})",
            risk_label, ctx.strategy, ctx.difficulty, tool_name, pipeline_decision.aggregate_score,
        ),
    );
    append_budget_to_output(&mut output, &state);
    append_latency(&mut output, start);
    let resp_json = serde_json::to_string(&output).unwrap_or_default();
    record_replay(
        &state,
        &session_id,
        "PreToolUse",
        &payload,
        &resp_json,
        start,
    )
    .await;
    Json(output)
}

/// POST /hooks/post-tool-use
async fn post_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    // Log the tool output for verification signals
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event("daemon", "post_tool_use", &payload)
        .await;

    // Post-tool-use: emit verification context based on what happened
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let response = input.tool_response.clone().unwrap_or_default();

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // Wire fatigue signals: record error or success
    let is_error = response_looks_like_error(&tool_name, &response);
    {
        let mut profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
        if is_error {
            profiler.on_error();
        } else {
            profiler.on_success();
        }
    }

    // Persist error/success counts into session context
    {
        let mut sess = session_ctx.lock().unwrap();
        if is_error {
            sess.errors += 1;
        } else {
            sess.success_count += 1;
        }
    }

    // Wire plasticity signals: if a recovery was injected and is pending,
    // record whether the outcome suggests success or failure.
    {
        use crate::profiler::plasticity::RecoveryOutcome;
        let mut tracker = state.plasticity.lock().expect("plasticity mutex poisoned");
        if tracker.has_pending_recovery() {
            if is_error {
                tracker.record_outcome(RecoveryOutcome::Failure);
            } else {
                tracker.record_outcome(RecoveryOutcome::Success);
            }
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

    let mut output = HookOutput::context("PostToolUse".to_string(), context.to_string());
    append_budget_to_output(&mut output, &state);
    append_latency(&mut output, start);
    let resp_json = serde_json::to_string(&output).unwrap_or_default();
    record_replay(
        &state,
        &session_id,
        "PostToolUse",
        &payload,
        &resp_json,
        start,
    )
    .await;

    // System 2: async post-processing (fire-and-forget)
    {
        let state_clone = state.clone();
        let session_clone = session_ctx.clone();
        let tool_name_clone = tool_name.clone();
        tokio::spawn(async move {
            crate::postprocess::after_post_tool_use(
                state_clone,
                session_clone,
                tool_name_clone,
                is_error,
            )
            .await;
        });
    }

    Json(output)
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
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state
        .memory
        .log_event("daemon", "user_prompt_submit", &payload)
        .await;

    // Wire fatigue signal: record the prompt
    let prompt_text = input.prompt.clone().unwrap_or_default();
    {
        let mut profiler = state.fatigue.lock().expect("fatigue mutex poisoned");
        profiler.on_prompt(&prompt_text, chrono::Utc::now());
    }

    // Estimate and consume tokens for the prompt (~4 chars per token, $0.000003/token)
    {
        let estimated_tokens = (prompt_text.len() / 4) as u64;
        let estimated_cost = estimated_tokens as f64 * 0.000003;
        let mut budget = state.budget.lock().expect("budget mutex poisoned");
        budget.consume(estimated_tokens, estimated_cost);
    }

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // Run ControlLoop stages 1-6 for analysis
    let mut ctx = LoopContext::new(input);
    state.control_loop.run_range(&mut ctx, 0, 6);

    // Persist control loop results into session context
    {
        let mut sess = session_ctx.lock().unwrap();
        sess.task_type = ctx.task_type;
        sess.risk = ctx.risk;
        sess.strategy = ctx.strategy;
        sess.difficulty = ctx.difficulty;
        sess.competence = ctx.competence;
    }

    let risk_label = match ctx.risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    };

    // Run TopologyPlanner to determine execution topology
    let task_type = ctx.task_type.unwrap_or(TaskType::Bugfix);
    let plan = TopologyPlanner::plan(ctx.risk, ctx.difficulty, task_type);

    // Store execution plan in session for use by the stop handler
    {
        let mut sess = session_ctx.lock().unwrap();
        sess.execution_plan = Some(plan.clone());
    }

    let strategy_label = format!("{:?}", ctx.strategy).to_lowercase();
    let strategy_kebab = strategy_label.replace('_', "-");
    let task_label = ctx
        .task_type
        .map(|t| format!("{:?}", t).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    let topology_label = format!("{:?}", plan.topology).to_lowercase();

    let readable = format_readable(
        risk_label,
        &strategy_kebab,
        ctx.budget.max_tokens,
        &task_label,
        &topology_label,
    );

    let mut output = HookOutput::context(
        "UserPromptSubmit".to_string(),
        format!("{} | Prompt analysed via control loop", readable),
    );
    append_budget_to_output(&mut output, &state);
    append_latency(&mut output, start);
    let resp_json = serde_json::to_string(&output).unwrap_or_default();
    record_replay(
        &state,
        &session_id,
        "UserPromptSubmit",
        &payload,
        &resp_json,
        start,
    )
    .await;

    // System 2: async post-processing (fire-and-forget)
    {
        let state_clone = state.clone();
        let session_clone = session_ctx.clone();
        tokio::spawn(async move {
            crate::postprocess::after_user_prompt_submit(state_clone, session_clone).await;
        });
    }

    Json(output)
}

/// POST /hooks/stop
///
/// Run ControlLoop stages 9-12 (calibrate, compact, decide, learn)
/// and return a proof-packet enforcement hint.
async fn stop(State(state): State<AppState>, Json(input): Json<HookInput>) -> Json<HookOutput> {
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "stop", &payload).await;

    // Completion verification: check Claude's claims match filesystem reality
    let last_msg = input.last_assistant_message.as_deref().unwrap_or("");
    let cwd = input.cwd.as_deref().unwrap_or(".");
    let verification = metaygn_verifiers::completion::verify_completion(last_msg, cwd);

    if !verification.verified {
        // BLOCK: Claude said "done" but files are missing
        let issues = verification.blocking_issues.join("; ");
        let mut output = HookOutput::context(
            "Stop".to_string(),
            format!(
                "COMPLETION CHECK FAILED: {}. Claude claimed completion but verification found issues. \
                 Review before accepting.",
                issues
            ),
        );
        append_budget_to_output(&mut output, &state);
        append_latency(&mut output, start);
        let resp_json = serde_json::to_string(&output).unwrap_or_default();
        record_replay(&state, &session_id, "Stop", &payload, &resp_json, start).await;
        return Json(output);
    }

    // Pruner analysis: check if the last assistant message shows repeated errors
    let pruner = ContextPruner::with_defaults();
    let last_msg_text = input.last_assistant_message.as_deref().unwrap_or("");
    let pruner_analysis = pruner.analyze(&[crate::proxy::pruner::Message {
        role: "assistant".to_string(),
        content: last_msg_text.to_string(),
    }]);

    // If the pruner detects errors, generate an amplified recovery message
    let recovery_note = if pruner_analysis.consecutive_errors > 0 {
        let reason = pruner_analysis
            .suggested_injection
            .as_deref()
            .unwrap_or("repeated errors detected");
        let level = state.plasticity.lock().unwrap().amplification_level();
        let recovery_msg = pruner.amplified_recovery(reason, level);
        state.plasticity.lock().unwrap().record_recovery_injected();
        Some(recovery_msg)
    } else {
        None
    };

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    let mut ctx = LoopContext::new(input);

    // Feed accumulated session data into the LoopContext so that stop-phase
    // stages (calibrate, compact, decide, learn) operate on the full session
    // picture rather than starting from scratch.
    {
        let sess = session_ctx.lock().unwrap();
        ctx.task_type = sess.task_type;
        ctx.risk = sess.risk;
        ctx.strategy = sess.strategy;
        ctx.difficulty = sess.difficulty;
        ctx.competence = sess.competence;
        ctx.verification_results = sess.verification_results.clone();
        ctx.lessons = sess.lessons.clone();
    }

    // Use the stored execution plan if available; otherwise fall back to
    // running stages 8-12 directly (backward-compatible path).
    let plan = {
        let sess = session_ctx.lock().unwrap();
        sess.execution_plan.clone()
    };
    let decision = if let Some(ref plan) = plan {
        state.control_loop.run_plan(&mut ctx, plan)
    } else {
        state.control_loop.run_range(&mut ctx, 8, 12)
    };

    let metacog = ctx.metacog_vector.compact_encode();
    let lessons_summary = if ctx.lessons.is_empty() {
        "none".to_string()
    } else {
        ctx.lessons.join("; ")
    };

    // Build base context from control loop
    let mut context_msg = format!(
        "[decision:{:?}] [metacog:{}] [lessons:{}] Proof packet enforcement recommended",
        decision, metacog, lessons_summary,
    );

    // Append recovery note if pruner detected errors
    if let Some(ref note) = recovery_note {
        context_msg.push_str(&format!(" {}", note));
    }

    // Append completion warnings if any
    if !verification.warnings.is_empty() {
        let warns = verification.warnings.join("; ");
        context_msg.push_str(&format!(" [completion_warnings: {}]", warns));
    }

    let mut output = HookOutput::context("Stop".to_string(), context_msg);
    append_budget_to_output(&mut output, &state);
    append_latency(&mut output, start);
    let resp_json = serde_json::to_string(&output).unwrap_or_default();
    record_replay(&state, &session_id, "Stop", &payload, &resp_json, start).await;

    // System 2: async post-processing (fire-and-forget)
    {
        let state_clone = state.clone();
        let session_clone = session_ctx.clone();
        let decision_str = format!("{:?}", decision);
        let lessons_clone = ctx.lessons.clone();
        tokio::spawn(async move {
            crate::postprocess::after_stop(state_clone, session_clone, decision_str, lessons_clone)
                .await;
        });
    }

    // Clean up session context now that the session is ending
    state.sessions.remove(&session_id);

    Json(output)
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
