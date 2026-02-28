use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 4: Determine whether a tool invocation is required.
pub struct ToolNeedStage;

impl Stage for ToolNeedStage {
    fn name(&self) -> &'static str {
        "tool_need"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // A tool is necessary when the hook input carries a tool name,
        // meaning Claude Code is about to invoke (or has just invoked) a tool.
        ctx.tool_necessary = ctx.input.tool_name.is_some();

        tracing::debug!(
            stage = self.name(),
            tool_necessary = ctx.tool_necessary,
            tool_name = ?ctx.input.tool_name,
            "assessed tool need"
        );

        StageResult::Continue
    }
}
