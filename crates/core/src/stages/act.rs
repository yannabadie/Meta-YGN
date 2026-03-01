use super::{Stage, StageResult};
use crate::context::{IntendedAction, LoopContext};

/// Stage 7: Record the intended action for post-verification.
///
/// Captures what tool is about to be used, its target, and the purpose
/// derived from the selected strategy. This information is used by
/// downstream stages (verify, calibrate) to assess execution outcomes.
pub struct ActStage;

impl Stage for ActStage {
    fn name(&self) -> &'static str {
        "act"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        if let Some(ref tool_input) = ctx.input.tool_input {
            let tool = ctx.input.tool_name.clone().unwrap_or_default();
            let target = tool_input
                .get("file_path")
                .or(tool_input.get("command"))
                .or(tool_input.get("path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let strategy_name = format!("{:?}", ctx.strategy);
            ctx.intended_action = Some(IntendedAction {
                tool,
                target,
                purpose: format!("Execute via {} strategy", strategy_name),
            });
        }
        StageResult::Continue
    }
}
