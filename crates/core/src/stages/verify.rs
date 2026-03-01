use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 8: Collect verification results from the tool response.
pub struct VerifyStage;

impl Stage for VerifyStage {
    fn name(&self) -> &'static str {
        "verify"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        ctx.verification_results.clear();

        // Check intended action against actual tool usage.
        if let Some(ref action) = ctx.intended_action {
            if let Some(ref tool_name) = ctx.input.tool_name {
                if *tool_name != action.tool && !action.tool.is_empty() {
                    ctx.verification_results.push(format!(
                        "tool_mismatch: intended '{}' but executed '{}'",
                        action.tool, tool_name
                    ));
                }
            }
        }

        // Parse test results from Bash tool responses.
        if let Some(ref tool_name) = ctx.input.tool_name {
            if tool_name == "Bash" {
                if let Some(ref response) = ctx.input.tool_response {
                    // Look for common test result patterns.
                    // Pattern: "N failed"
                    let lower = response.to_lowercase();
                    if let Some(pos) = lower.find("failed") {
                        // Extract the number before "failed".
                        let prefix = &lower[..pos];
                        if let Some(num_str) = prefix.split_whitespace().last() {
                            if let Ok(failed) = num_str.parse::<u32>() {
                                if failed > 0 {
                                    ctx.verification_results.push(format!(
                                        "test_failures: {} tests failed",
                                        failed
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // If we have an error from a previous tool invocation, record it.
        if let Some(err) = &ctx.input.error {
            ctx.verification_results.push(format!("tool_error: {err}"));
        }

        // If we have a tool response, do basic sanity checks.
        if let Some(response) = &ctx.input.tool_response {
            let response: &str = response.as_str();
            if response.is_empty() {
                ctx.verification_results
                    .push("warning: empty tool response".to_string());
            } else {
                ctx.verification_results
                    .push(format!("tool_response_length: {}", response.len()));
            }

            // Check for common error patterns in tool output.
            let lower = response.to_lowercase();
            let error_patterns = ["error", "failed", "exception", "panic", "traceback"];
            for pattern in &error_patterns {
                if lower.contains(pattern) {
                    ctx.verification_results
                        .push(format!("response_contains: {pattern}"));
                }
            }
        } else {
            ctx.verification_results
                .push("no_tool_response".to_string());
        }

        tracing::debug!(
            stage = self.name(),
            results = ?ctx.verification_results,
            "verification complete"
        );

        StageResult::Continue
    }
}
