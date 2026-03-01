use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 10: Memory compaction.
///
/// Deduplicates lessons (keeping unique entries, max 5) and generates
/// a compact summary of the current loop iteration for downstream
/// consumption and archival.
pub struct CompactStage;

impl Stage for CompactStage {
    fn name(&self) -> &'static str {
        "compact"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // Deduplicate lessons (keep unique, max 5)
        let mut unique: Vec<String> = Vec::new();
        for lesson in &ctx.lessons {
            if !unique.iter().any(|l| l == lesson) && unique.len() < 5 {
                unique.push(lesson.clone());
            }
        }

        // Generate compact summary
        let summary = format!(
            "[compact] task={:?} risk={:?} strategy={:?} verifications={}/{}",
            ctx.task_type,
            ctx.risk,
            ctx.strategy,
            ctx.verification_results
                .iter()
                .filter(|r| !r.contains("error") && !r.contains("fail"))
                .count(),
            ctx.verification_results.len(),
        );
        unique.push(summary);
        ctx.lessons = unique;

        StageResult::Continue
    }
}
