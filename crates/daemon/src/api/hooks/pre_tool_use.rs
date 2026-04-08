use axum::extract::State;
use axum::response::Json;

use metaygn_core::context::LoopContext;
use metaygn_shared::protocol::{HookInput, HookOutput, PermissionDecision};
use metaygn_shared::state::RiskLevel;

use crate::app_state::AppState;

use super::{
    append_budget_to_output, append_latency, append_session_budget, extract_command, record_replay,
};

/// POST /hooks/pre-tool-use
///
/// 1. Run the GuardPipeline on tool_name + command
/// 2. If the pipeline blocks -> return deny (score 0) or ask (score > 0)
/// 3. Run ControlLoop stages 1-6 (classify through strategy) for risk assessment
/// 4. Return enriched context with risk level and strategy recommendation
pub(crate) async fn pre_tool_use(
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
        .log_event(&session_id, "pre_tool_use", &payload)
        .await;

    // Step 1: Run the guard pipeline
    let pipeline_decision = state.guard_pipeline.check(&tool_name, &command);

    if !pipeline_decision.allowed {
        // Record guard hit in adaptive guard memory
        if let Some(ref guard_name) = pipeline_decision.blocking_guard
            && let Ok(mut mem) = state.guard_memory.lock()
        {
            mem.record_hit(guard_name);
        }

        // Score 0 = destructive -> deny; score > 0 = high-risk -> ask
        let decision = if pipeline_decision.aggregate_score == 0 {
            PermissionDecision::Deny
        } else {
            // "Ask" means the user will see a confirmation dialog.
            // Track this guard block in the session so we can provide
            // false-positive feedback at session end if the session succeeds.
            if let Some(ref guard_name) = pipeline_decision.blocking_guard {
                let session_ctx = state.sessions.get_or_create(&session_id);
                if let Ok(mut sess) = session_ctx.lock() {
                    sess.guard_blocks.push(guard_name.clone());
                }
            }
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

        // Auto-checkpoint: save recovery point before destructive operation
        // NOTE: checkpoint functions use blocking I/O (std::process::Command,
        // std::fs::copy), so they are offloaded to spawn_blocking to avoid
        // starving the tokio runtime.
        let mut checkpoint_message: Option<String> = None;

        // Git checkpoint for git-destructive operations
        if command.contains("git")
            && (command.contains("reset")
                || command.contains("checkout")
                || command.contains("push")
                || command.contains("rebase"))
        {
            let cwd_owned = input.cwd.clone().unwrap_or_else(|| ".".to_string());
            checkpoint_message = tokio::task::spawn_blocking(move || {
                let cp = metaygn_verifiers::checkpoint::git_checkpoint(&cwd_owned);
                if cp.created {
                    tracing::info!(
                        checkpoint_type = ?cp.checkpoint_type,
                        location = %cp.location,
                        "auto-checkpoint created before git operation"
                    );
                    Some(cp.message)
                } else {
                    None
                }
            })
            .await
            .unwrap_or(None);
        }

        // File checkpoint for file-destructive operations
        if checkpoint_message.is_none()
            && (command.contains("rm")
                || command.contains("unlink")
                || command.contains("delete"))
        {
            let cwd_owned = input.cwd.clone().unwrap_or_else(|| ".".to_string());
            let command_owned = command.clone();
            checkpoint_message = tokio::task::spawn_blocking(move || {
                let target_files =
                    metaygn_verifiers::checkpoint::extract_target_files(&command_owned);
                if target_files.is_empty() {
                    return None;
                }
                let file_refs: Vec<&str> =
                    target_files.iter().map(|s| s.as_str()).collect();
                let cp =
                    metaygn_verifiers::checkpoint::file_checkpoint(&cwd_owned, &file_refs);
                if cp.created {
                    tracing::info!(
                        files_saved = cp.files_saved,
                        location = %cp.location,
                        "auto-checkpoint created before file deletion"
                    );
                    Some(cp.message)
                } else {
                    None
                }
            })
            .await
            .unwrap_or(None);
        }

        // Include recovery instructions in the response
        if let Some(ref cp_msg) = checkpoint_message {
            if let Some(ref mut hso) = output.hook_specific_output {
                match hso.additional_context {
                    Some(ref mut ctx) => {
                        ctx.push_str(&format!(" [checkpoint] {}", cp_msg));
                    }
                    None => {
                        hso.additional_context =
                            Some(format!("[checkpoint] {}", cp_msg));
                    }
                }
            }
        }

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
        // Combine the original prompt AND the command so that injection
        // markers in the prompt are not lost when we inject the command
        // text for risk assessment.
        let original_prompt = input_for_loop.prompt.clone().unwrap_or_default();
        input_for_loop.prompt = Some(format!("{} {}", original_prompt, cmd));
    }
    let mut ctx = LoopContext::new(input_for_loop);

    // Semantic routing: classify the command for tiered verification.
    // Only runs when the semantic feature is enabled AND a real embedding
    // provider is available (hash embeddings are gated out at init time).
    #[cfg(feature = "semantic")]
    if let Some(ref router) = state.router {
        let task_context = if let Ok(sess) = session_ctx.lock() {
            sess.task_type.map(|t| format!("{:?}", t))
        } else {
            None
        };
        let hint = router.routing_hint(&cmd, task_context.as_deref());
        tracing::info!(
            command = %cmd,
            routing_hint = ?hint,
            "semantic router classification"
        );
        ctx.routing_hint = Some(hint);
    }

    state.control_loop.run_range(&mut ctx, 0, 6);

    // Evolver influence: if the best heuristic version has learned that
    // the risk markers relevant to this command have lower weights than
    // default thresholds, downgrade the risk from High to Medium.
    // This closes the feedback loop: evolver outcomes -> risk assessment.
    if ctx.risk == RiskLevel::High
        && let Ok(evolver) = state.evolver.lock()
        && let Some(best) = evolver.best()
    {
        let cmd_lower = cmd.to_lowercase();
        let marker_checks: &[(&str, &str)] = &[
            ("exec_command", ""),        // Bash tool itself
            ("fs_write", "write"),
            ("network_access", "curl"),
            ("env_mutation", "export"),
            ("credential_access", "credential"),
            ("large_diff", "large"),
            ("multi_file", "multi"),
        ];
        let relevant_weights: Vec<f64> = marker_checks
            .iter()
            .filter(|(_, keyword)| keyword.is_empty() || cmd_lower.contains(keyword))
            .filter_map(|(marker, _)| best.risk_weights.get(*marker))
            .copied()
            .collect();

        // If the evolver has learned that relevant risk markers
        // are all low-weight (< 0.4), downgrade from High to Medium.
        if !relevant_weights.is_empty() {
            let avg_weight: f64 =
                relevant_weights.iter().sum::<f64>() / relevant_weights.len() as f64;
            if avg_weight < 0.4 {
                tracing::info!(
                    avg_weight = avg_weight,
                    generation = best.generation,
                    "evolver influence: downgrading risk High → Medium (learned low weights)"
                );
                ctx.risk = RiskLevel::Medium;
            }
        }
    }

    // Persist tool call count, check sequence alerts, and MOP meltdown in one lock
    let (sequence_warning, meltdown_warning) = if let Ok(mut sess) = session_ctx.lock() {
        sess.tool_calls += 1;

        let seq_warn = {
            let alerts = sess.sequence_monitor.alerts();
            if !alerts.is_empty() {
                let latest = &alerts[alerts.len() - 1];
                Some(format!(
                    " [sequence_alert:{}] {}",
                    latest.rule, latest.description
                ))
            } else {
                None
            }
        };

        let melt_warn = if sess.mop_detector.is_melting_down() {
            Some(" [MELTDOWN] Agent behavioral collapse detected — consider escalating".to_string())
        } else {
            None
        };

        (seq_warn, melt_warn)
    } else {
        tracing::warn!("session mutex poisoned — skipping tool_calls/sequence/meltdown check");
        (None, None)
    };

    let risk_label = match ctx.risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
    };

    // Return enriched context with risk + strategy
    let mut context_msg = format!(
        "[risk:{}] [strategy:{:?}] [difficulty:{:.2}] Tool '{}' allowed by guard pipeline (score:{})",
        risk_label, ctx.strategy, ctx.difficulty, tool_name, pipeline_decision.aggregate_score,
    );
    if let Some(ref warning) = sequence_warning {
        context_msg.push_str(warning);
    }
    if let Some(ref warning) = meltdown_warning {
        context_msg.push_str(warning);
    }
    let mut output = HookOutput::context("PreToolUse".to_string(), context_msg);
    append_session_budget(&mut output, &session_ctx);
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
