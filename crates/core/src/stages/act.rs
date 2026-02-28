use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 7: Execute the action (no-op placeholder).
///
/// Actual execution happens outside the control loop -- the loop only
/// decides *what* to do and *how*. This stage exists as a pipeline
/// checkpoint where future execution hooks can be inserted.
pub struct ActStage;

impl Stage for ActStage {
    fn name(&self) -> &'static str {
        "act"
    }

    fn run(&self, _ctx: &mut LoopContext) -> StageResult {
        tracing::debug!(stage = self.name(), "act stage (no-op)");
        StageResult::Continue
    }
}
