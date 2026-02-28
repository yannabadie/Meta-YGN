use super::{Stage, StageResult};
use crate::context::LoopContext;

/// Stage 9: Adjust the metacognitive vector based on verification results.
pub struct CalibrateStage;

impl Stage for CalibrateStage {
    fn name(&self) -> &'static str {
        "calibrate"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        let v = &mut ctx.metacog_vector;

        // Count error signals from verification.
        let error_count = ctx
            .verification_results
            .iter()
            .filter(|r| {
                r.starts_with("tool_error")
                    || r.starts_with("response_contains")
                    || r.contains("empty tool response")
            })
            .count();

        if error_count == 0 {
            // No errors: boost confidence and grounding.
            v.confidence = (v.confidence + 0.1).min(1.0);
            v.grounding = (v.grounding + 0.1).min(1.0);
        } else {
            // Errors detected: reduce confidence proportionally.
            let penalty = (error_count as f64 * 0.15).min(0.5);
            v.confidence = (v.confidence - penalty).max(0.0);
            v.grounding = (v.grounding - 0.1).max(0.0);
        }

        // Complexity tracks the difficulty estimate.
        v.complexity = ctx.difficulty as f64;

        // Coherence: higher when we have a classified task type.
        if ctx.task_type.is_some() {
            v.coherence = (v.coherence + 0.1).min(1.0);
        }

        // Progress: bump slightly for each stage that ran.
        v.progress = (v.progress + 0.1).min(1.0);

        tracing::debug!(
            stage = self.name(),
            confidence = v.confidence,
            coherence = v.coherence,
            grounding = v.grounding,
            overall = v.overall_quality(),
            "calibrated metacog vector"
        );

        StageResult::Continue
    }
}
