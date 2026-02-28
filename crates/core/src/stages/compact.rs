use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 10: Memory compaction placeholder.
///
/// In a future iteration this stage will summarise working memory and
/// archive stale entries to cold storage. For now it is a no-op.
pub struct CompactStage;

impl Stage for CompactStage {
    fn name(&self) -> &'static str {
        "compact"
    }

    fn run(&self, _ctx: &mut LoopContext) -> StageResult {
        tracing::debug!(stage = self.name(), "compact stage (no-op)");
        StageResult::Continue
    }
}
