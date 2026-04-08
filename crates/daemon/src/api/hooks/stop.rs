use axum::extract::State;
use axum::response::Json;

use metaygn_core::context::LoopContext;
use metaygn_shared::protocol::{HookInput, HookOutput};

use crate::app_state::AppState;
use crate::proxy::pruner::ContextPruner;

use super::{append_budget_to_output, append_latency, append_session_budget, record_replay};

/// POST /hooks/stop
///
/// Run ControlLoop stages 9-12 (calibrate, compact, decide, learn)
/// and return a proof-packet enforcement hint.
pub(crate) async fn stop(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let start = std::time::Instant::now();
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event(&session_id, "stop", &payload).await;

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

    // Get or create session context for cross-hook state persistence
    let session_ctx = state.sessions.get_or_create(&session_id);

    // If the pruner detects errors, generate an amplified recovery message
    // using the session-local plasticity tracker. Also update global tracker.
    let recovery_note = if pruner_analysis.consecutive_errors > 0 {
        let reason = pruner_analysis
            .suggested_injection
            .as_deref()
            .unwrap_or("repeated errors detected");
        let level = if let Ok(sess) = session_ctx.lock() {
            sess.plasticity.amplification_level()
        } else {
            tracing::warn!("session mutex poisoned — using default amplification level");
            0
        };
        let recovery_msg = pruner.amplified_recovery(reason, level);
        if let Ok(mut sess) = session_ctx.lock() {
            sess.plasticity.record_recovery_injected();
        } else {
            tracing::warn!("session mutex poisoned — skipping session plasticity record_recovery_injected");
        }
        if let Ok(mut tracker) = state.plasticity.lock() {
            tracker.record_recovery_injected();
        } else {
            tracing::warn!("plasticity mutex poisoned — skipping global plasticity record_recovery_injected");
        }
        Some(recovery_msg)
    } else {
        None
    };

    let mut ctx = LoopContext::new(input);

    // Feed accumulated session data into the LoopContext so that stop-phase
    // stages (calibrate, compact, decide, learn) operate on the full session
    // picture rather than starting from scratch.
    {
        if let Ok(sess) = session_ctx.lock() {
            ctx.task_type = sess.task_type;
            ctx.risk = sess.risk;
            ctx.strategy = sess.strategy;
            ctx.difficulty = sess.difficulty;
            ctx.competence = sess.competence;
            ctx.verification_results = sess.verification_results.clone();
            ctx.lessons = sess.lessons.clone();
        } else {
            tracing::warn!("session mutex poisoned — skipping session data feed into LoopContext");
        }
    }

    // Load historical competence before running finalization stages
    if let Some(task_type) = ctx.task_type {
        let task_type_str = format!("{:?}", task_type);
        if let Ok(Some(rate)) = state
            .memory
            .success_rate_for_task_type(&task_type_str, 20, 5)
            .await
        {
            ctx.historical_success_rate = Some(rate);
        }
    }

    // Always run finalization stages regardless of topology.
    // This fixes the bug where Research/Trivial topologies skipped
    // calibrate/compact/decide/learn.
    let decision = state.control_loop.run_finalization(&mut ctx);

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
    append_session_budget(&mut output, &session_ctx);
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

    // Adaptive guard feedback: if the session had guard blocks (rules that
    // triggered "ask") and completed with few errors, those blocks were likely
    // false positives — the user overrode them and the session still succeeded.
    {
        let guard_blocks = if let Ok(sess) = session_ctx.lock() {
            sess.guard_blocks.clone()
        } else {
            Vec::new()
        };
        if !guard_blocks.is_empty() {
            let was_successful = ctx.verification_results.is_empty()
                && session_ctx
                    .lock()
                    .map(|s| s.errors <= 1)
                    .unwrap_or(false);
            if let Ok(mut mem) = state.guard_memory.lock() {
                for rule in &guard_blocks {
                    // Successful session -> guard block was likely a false positive.
                    // Failed session -> guard was probably right (true positive).
                    mem.record_outcome(rule, !was_successful);
                }
            }
            tracing::info!(
                rules = ?guard_blocks,
                was_successful = was_successful,
                "adaptive guard feedback recorded"
            );
        }
    }

    // Clean up session context now that the session is ending
    state.sessions.remove(&session_id);

    Json(output)
}
