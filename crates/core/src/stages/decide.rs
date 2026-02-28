use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::{Decision, RiskLevel};

/// Stage 11: Make the final decision for this loop iteration.
pub struct DecideStage;

/// Competence threshold below which high-risk tasks are escalated.
const ESCALATION_COMPETENCE_THRESHOLD: f32 = 0.4;

/// Metacognitive quality threshold below which we revise rather than continue.
const REVISE_QUALITY_THRESHOLD: f64 = 0.3;

impl Stage for DecideStage {
    fn name(&self) -> &'static str {
        "decide"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        let quality = ctx.metacog_vector.overall_quality();

        // High risk + low competence => escalate to human.
        if ctx.risk == RiskLevel::High && ctx.competence < ESCALATION_COMPETENCE_THRESHOLD {
            ctx.decision = Decision::Escalate;
            let reason = format!(
                "high risk ({:?}) with low competence ({:.2})",
                ctx.risk, ctx.competence
            );
            tracing::warn!(stage = self.name(), %reason, "escalating");
            return StageResult::Escalate(reason);
        }

        // Very low metacognitive quality => revise.
        if quality < REVISE_QUALITY_THRESHOLD {
            ctx.decision = Decision::Revise;
            tracing::debug!(
                stage = self.name(),
                quality,
                "quality below threshold, revising"
            );
            return StageResult::Continue;
        }

        // Has errors in verification => revise.
        let has_errors = ctx
            .verification_results
            .iter()
            .any(|r| r.starts_with("tool_error") || r.starts_with("response_contains"));
        if has_errors {
            ctx.decision = Decision::Revise;
            tracing::debug!(stage = self.name(), "verification errors detected, revising");
            return StageResult::Continue;
        }

        // Default: continue.
        ctx.decision = Decision::Continue;

        tracing::debug!(
            stage = self.name(),
            decision = ?ctx.decision,
            quality,
            "decided"
        );

        StageResult::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escalation_threshold() {
        assert!(ESCALATION_COMPETENCE_THRESHOLD > 0.0);
        assert!(ESCALATION_COMPETENCE_THRESHOLD < 1.0);
    }
}
