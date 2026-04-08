mod post_tool_use;
mod pre_tool_use;
mod routes;
mod session_end;
mod stop;
mod user_prompt_submit;

use metaygn_shared::protocol::{HookOutput, HookSpecificOutput};

use crate::app_state::AppState;

pub use routes::routes;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Append the global budget summary to a HookOutput's additionalContext.
/// Used only for early-return paths where no session context exists yet.
pub(crate) fn append_budget_to_output(output: &mut HookOutput, state: &AppState) {
    if let Ok(budget) = state.budget.lock() {
        let summary = budget.summary();
        append_budget_summary(output, summary);
    } else {
        tracing::warn!("budget mutex poisoned — skipping budget display");
    }
}

/// Append the session-local budget summary to a HookOutput's additionalContext.
/// Preferred over `append_budget_to_output` wherever a session context is available,
/// so that budget display reflects the current session rather than global state.
pub(crate) fn append_session_budget(
    output: &mut HookOutput,
    session: &std::sync::Arc<std::sync::Mutex<crate::session::SessionContext>>,
) {
    if let Ok(sess) = session.lock() {
        let summary = sess.budget.summary();
        append_budget_summary(output, summary);
    } else {
        tracing::warn!("session mutex poisoned — skipping session budget display");
    }
}

/// Shared helper: write a budget summary string into a HookOutput.
fn append_budget_summary(output: &mut HookOutput, summary: String) {
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
            output.hook_specific_output = Some(HookSpecificOutput {
                additional_context: Some(summary),
                ..Default::default()
            });
        }
    }
}

/// Extract the command string from tool_input. The command may be in
/// tool_input.command (Bash tool) or tool_input itself might be a string.
pub(crate) fn extract_command(input: &metaygn_shared::protocol::HookInput) -> String {
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
pub(crate) fn format_readable(
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
pub(crate) fn append_latency(output: &mut HookOutput, start: std::time::Instant) {
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
pub(crate) async fn record_replay(
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

/// Check whether a command string looks like `git push`.
pub(crate) fn command_looks_like_git_push(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("git push") || lower.contains("git push --force")
}

/// Determine whether a tool response indicates an error.
pub(crate) fn response_looks_like_error(tool_name: &str, response: &str) -> bool {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("error")
        || lower.starts_with("panic")
        || lower.starts_with("fatal")
        || lower.contains("traceback")
    {
        return true;
    }

    if tool_name == "Bash" {
        let explicit_failure = lower.contains("fail")
            || lower.contains("failed")
            || lower.contains("error:")
            || lower.contains("command not found")
            || lower.contains("permission denied")
            || lower.contains("exit code");

        // Avoid false positives for common success summaries that mention "failed".
        let known_success = lower.contains("0 failed")
            || lower.contains("no failures")
            || lower.contains("all tests passed")
            || lower.contains("test result: ok");

        return explicit_failure && !known_success;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::response_looks_like_error;

    #[test]
    fn bash_failures_are_detected_case_insensitively() {
        assert!(response_looks_like_error("Bash", "FAIL: 2 tests"));
        assert!(response_looks_like_error("Bash", "error: command failed"));
        assert!(response_looks_like_error("Bash", "Command not found: foo"));
    }

    #[test]
    fn bash_success_summaries_are_not_false_positives() {
        assert!(!response_looks_like_error(
            "Bash",
            "test result: ok. 12 passed; 0 failed"
        ));
        assert!(!response_looks_like_error(
            "Bash",
            "All tests passed. No failures."
        ));
    }

    #[test]
    fn generic_error_prefixes_are_detected() {
        assert!(response_looks_like_error("Write", "Error: invalid JSON"));
        assert!(response_looks_like_error("Write", "panic: unreachable"));
        assert!(!response_looks_like_error(
            "Write",
            "Completed successfully"
        ));
    }
}
