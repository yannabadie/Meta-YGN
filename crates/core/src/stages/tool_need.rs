use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 4: Determine whether a tool invocation is required.
///
/// Sets `tool_necessary` and `tool_necessity_reason` on the context.
/// Advisory only — never blocks tool execution.
pub struct ToolNeedStage;

/// Check if a Bash command is purely display (echo/printf with no side-effects).
fn is_echo_only(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    // Conservative: rejects if special chars appear anywhere, even inside quotes.
    // False negatives (missing an echo-only) are safe; false positives would be wrong.
    if (trimmed.starts_with("echo ") || trimmed.starts_with("printf "))
        && !trimmed.contains('>')
        && !trimmed.contains('|')
        && !trimmed.contains('&')
        && !trimmed.contains(';')
    {
        return true;
    }
    false
}

impl Stage for ToolNeedStage {
    fn name(&self) -> &'static str {
        "tool_need"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        ctx.tool_necessary = ctx.input.tool_name.is_some();

        if ctx.tool_necessary {
            let tool = ctx.input.tool_name.as_deref().unwrap_or("");
            let cmd = ctx
                .input
                .tool_input
                .as_ref()
                .and_then(|ti| ti.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            ctx.tool_necessity_reason = match tool {
                "Bash" if is_echo_only(cmd) => {
                    ctx.tool_necessary = false;
                    Some("echo/printf without side-effects — tool not necessary".into())
                }
                _ => Some("tool invocation detected".into()),
            };
        } else {
            ctx.tool_necessity_reason = None;
        }

        tracing::debug!(
            stage = self.name(),
            tool_necessary = ctx.tool_necessary,
            tool_name = ?ctx.input.tool_name,
            reason = ?ctx.tool_necessity_reason,
            "assessed tool need"
        );

        StageResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::LoopContext;
    use metaygn_shared::protocol::HookInput;

    fn make_ctx(tool_name: Option<&str>, command: Option<&str>) -> LoopContext {
        let mut input = HookInput::default();
        input.tool_name = tool_name.map(|s| s.to_string());
        if let Some(cmd) = command {
            let mut map = serde_json::Map::new();
            map.insert(
                "command".to_string(),
                serde_json::Value::String(cmd.to_string()),
            );
            input.tool_input = Some(serde_json::Value::Object(map));
        }
        LoopContext::new(input)
    }

    #[test]
    fn echo_only_detected_as_unnecessary() {
        let mut ctx = make_ctx(Some("Bash"), Some("echo hello world"));
        ToolNeedStage.run(&mut ctx);
        assert!(!ctx.tool_necessary);
        assert!(
            ctx.tool_necessity_reason
                .as_deref()
                .unwrap()
                .contains("echo")
        );
    }

    #[test]
    fn cargo_test_is_necessary() {
        let mut ctx = make_ctx(Some("Bash"), Some("cargo test --workspace"));
        ToolNeedStage.run(&mut ctx);
        assert!(ctx.tool_necessary);
    }

    #[test]
    fn no_tool_name_is_not_necessary() {
        let mut ctx = make_ctx(None, None);
        ToolNeedStage.run(&mut ctx);
        assert!(!ctx.tool_necessary);
    }

    #[test]
    fn echo_with_redirect_is_necessary() {
        let mut ctx = make_ctx(Some("Bash"), Some("echo secret > leak.txt"));
        ToolNeedStage.run(&mut ctx);
        assert!(ctx.tool_necessary);
    }
}
