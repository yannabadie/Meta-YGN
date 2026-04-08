use axum::extract::State;
use axum::response::Json;

use metaygn_shared::protocol::{HookInput, HookOutput};

use crate::app_state::AppState;

use super::{
    append_latency, append_session_budget, command_looks_like_git_push, record_replay,
    response_looks_like_error,
};

/// POST /hooks/post-tool-use
pub(crate) async fn post_tool_use(
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
        .log_event(&session_id, "post_tool_use", &payload)
        .await;

    // Post-tool-use: emit verification context based on what happened
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let response = input.tool_response.clone().unwrap_or_default();

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // Wire fatigue and plasticity signals in a single session lock.
    // Session-local prevents cross-session bleed; global feeds /profiler endpoints.
    let is_error = response_looks_like_error(&tool_name, &response);
    {
        use crate::profiler::plasticity::RecoveryOutcome;
        if let Ok(mut sess) = session_ctx.lock() {
            if is_error {
                sess.fatigue.on_error();
                sess.errors += 1;
            } else {
                sess.fatigue.on_success();
                sess.success_count += 1;
            }
            // Sync session budget tokens
            sess.tokens_consumed = sess.budget.consumed_tokens();
            // Plasticity
            if sess.plasticity.has_pending_recovery() {
                if is_error {
                    sess.plasticity.record_outcome(RecoveryOutcome::Failure);
                } else {
                    sess.plasticity.record_outcome(RecoveryOutcome::Success);
                }
            }
        } else {
            tracing::warn!("session mutex poisoned — skipping fatigue/plasticity in post_tool_use");
        }
    }
    // Global fatigue and plasticity (separate mutexes, not session_ctx)
    {
        use crate::profiler::plasticity::RecoveryOutcome;
        if let Ok(mut profiler) = state.fatigue.lock() {
            if is_error {
                profiler.on_error();
            } else {
                profiler.on_success();
            }
        } else {
            tracing::warn!("fatigue mutex poisoned — skipping global fatigue update");
        }
        if let Ok(mut tracker) = state.plasticity.lock() {
            if tracker.has_pending_recovery() {
                if is_error {
                    tracker.record_outcome(RecoveryOutcome::Failure);
                } else {
                    tracker.record_outcome(RecoveryOutcome::Success);
                }
            }
        } else {
            tracing::warn!("plasticity mutex poisoned — skipping global plasticity update");
        }
    }

    // Tier 1 verification: validate config files in-process.
    // Collect verification results to push in a single lock below.
    let mut verification_context = String::new();
    let mut pending_verification_results: Vec<String> = Vec::new();
    if (tool_name == "Write" || tool_name == "Edit")
        && let Some(ref tool_input) = input.tool_input
        && let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str())
        && tool_name == "Write"
        && let Some(content) = tool_input.get("content").and_then(|v| v.as_str())
        && let Some(error) = crate::verification::validate_file_content(file_path, content)
    {
        verification_context = format!(" SYNTAX ERROR in {}: {}", file_path, error);
        pending_verification_results.push(format!("syntax_error: {} — {}", file_path, error));
    }

    // Tier 1.5 verification: tree-sitter syntax check for code files
    #[cfg(feature = "syntax")]
    if tool_name == "Write" || tool_name == "Edit" {
        if let Some(ref tool_input) = input.tool_input {
            if let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str()) {
                let ext = file_path.rsplit('.').next().unwrap_or("");
                if let Some(content) = tool_input.get("content").and_then(|v| v.as_str()) {
                    let errors = metaygn_verifiers::syntax::check_syntax(content, ext);
                    if !errors.is_empty() {
                        let detail = errors
                            .iter()
                            .map(|e| format!("L{}: {}", e.line, e.message))
                            .collect::<Vec<_>>()
                            .join(", ");
                        verification_context.push_str(&format!(
                            " SYNTAX: {} error(s) in {}: {}",
                            errors.len(),
                            file_path,
                            detail
                        ));
                        pending_verification_results
                            .push(format!("syntax_error: {} — {}", file_path, detail));
                    }
                }
            }
        }
    }

    // Sequence monitor + MOP detection + deferred verification results: single lock
    {
        use metaygn_core::sequence_monitor::{ActionState, ActionType, TargetType};

        let action_type = if is_error {
            ActionType::Error
        } else {
            match tool_name.as_str() {
                "Write" | "Edit" | "MultiEdit" => ActionType::Write,
                "Read" | "Glob" | "Grep" => ActionType::Read,
                "Bash" => {
                    let resp_lower = response.to_lowercase();
                    if resp_lower.contains("git push") || command_looks_like_git_push(&response) {
                        ActionType::GitPush
                    } else {
                        ActionType::Execute
                    }
                }
                name if name.starts_with("mcp__") => ActionType::NetworkRead,
                _ => ActionType::Unknown,
            }
        };

        let file_path = input
            .tool_input
            .as_ref()
            .and_then(|ti| ti.get("file_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let target_type = if file_path.contains("test")
            || file_path.contains("_test")
            || file_path.contains("spec")
        {
            TargetType::TestFile
        } else if file_path.contains(".env")
            || file_path.contains("secret")
            || file_path.contains("credential")
            || file_path.contains(".pem")
            || file_path.contains(".key")
        {
            TargetType::SensitivePath
        } else if tool_name.starts_with("mcp__") {
            TargetType::Url
        } else {
            TargetType::File
        };

        if let Ok(mut sess) = session_ctx.lock() {
            // Push deferred verification results
            sess.verification_results
                .extend(pending_verification_results);

            // Sequence monitor
            let alert_count_before = sess.sequence_monitor.alerts().len();
            sess.sequence_monitor.record(ActionState {
                action_type,
                target_type,
                tool_name: tool_name.clone(),
                detail: file_path.to_string(),
            });
            let alerts = sess.sequence_monitor.alerts();
            if alerts.len() > alert_count_before {
                let latest = &alerts[alerts.len() - 1];
                tracing::warn!(
                    rule = %latest.rule,
                    description = %latest.description,
                    "SEQUENCE ALERT: dangerous multi-action pattern detected"
                );
            }

            // MOP detection
            let report = sess.mop_detector.record(&tool_name);
            if report.meltdown_detected {
                tracing::warn!(
                    entropy = report.entropy,
                    step = report.meltdown_step,
                    repetition_ratio = report.repetition_ratio,
                    "MELTDOWN DETECTED: agent behavioral collapse onset"
                );
            }
        }
    }

    let context = if tool_name == "Bash" && is_error {
        "Error detected in Bash output. Review results before proceeding."
    } else if tool_name == "Write" || tool_name == "Edit" {
        "File modification recorded. Verify changes align with intent."
    } else if tool_name.starts_with("mcp__") {
        "MCP tool output recorded. Verify external system state."
    } else {
        "Tool output recorded."
    };

    let mut output = HookOutput::context(
        "PostToolUse".to_string(),
        format!("{}{}", context, verification_context),
    );
    append_session_budget(&mut output, &session_ctx);
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
        let file_path_clone = input
            .tool_input
            .as_ref()
            .and_then(|ti| ti.get("file_path"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        tokio::spawn(async move {
            crate::postprocess::after_post_tool_use(
                state_clone,
                session_clone,
                tool_name_clone,
                is_error,
                file_path_clone,
            )
            .await;
        });
    }

    Json(output)
}
