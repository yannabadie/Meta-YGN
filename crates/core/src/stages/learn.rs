use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 12: Collect lessons learned from the entire pipeline run.
pub struct LearnStage;

impl Stage for LearnStage {
    fn name(&self) -> &'static str {
        "learn"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // Summarise what happened in this loop iteration.
        if let Some(tt) = ctx.task_type {
            ctx.lessons
                .push(format!("task_type={tt:?}"));
        }
        ctx.lessons
            .push(format!("risk={:?} difficulty={:.2}", ctx.risk, ctx.difficulty));
        ctx.lessons
            .push(format!("strategy={:?}", ctx.strategy));
        ctx.lessons
            .push(format!("decision={:?}", ctx.decision));
        ctx.lessons.push(format!(
            "metacog_quality={:.3}",
            ctx.metacog_vector.overall_quality()
        ));

        // Record verification issues as lessons.
        for result in &ctx.verification_results {
            if result.starts_with("tool_error") || result.starts_with("response_contains") {
                ctx.lessons
                    .push(format!("verification_issue: {result}"));
            }
        }

        tracing::debug!(
            stage = self.name(),
            lesson_count = ctx.lessons.len(),
            "collected lessons"
        );

        StageResult::Continue
    }
}
