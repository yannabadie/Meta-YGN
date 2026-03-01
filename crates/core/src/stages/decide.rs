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

        // Overconfidence detected (EGPO) → revise with warning
        if ctx.overconfidence_score > 0.3 {
            ctx.decision = Decision::Revise;
            tracing::warn!(
                stage = self.name(),
                overconfidence_score = ctx.overconfidence_score,
                "overconfidence detected (EGPO), forcing revise"
            );
            return StageResult::Continue;
        }

        // Plasticity lost (RL2F) → escalate
        if ctx.plasticity_lost {
            ctx.decision = Decision::Escalate;
            tracing::warn!(
                stage = self.name(),
                "plasticity lost — model ignoring recovery feedback, escalating"
            );
            return StageResult::Escalate(
                "plasticity lost: model ignoring recovery feedback after multiple attempts".into(),
            );
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
            .any(|r| {
                r.starts_with("tool_error")
                    || r.starts_with("response_contains")
                    || r.starts_with("test_failures")
                    || r.starts_with("tool_mismatch")
                    || r.starts_with("syntax_error")
            });
        if has_errors {
            ctx.decision = Decision::Revise;
            tracing::debug!(
                stage = self.name(),
                "verification errors detected, revising"
            );
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
    use crate::context::LoopContext;
    use metaygn_shared::protocol::{HookEvent, HookInput};

    fn make_input() -> HookInput {
        HookInput {
            hook_event_name: HookEvent::PreToolUse,
            session_id: None,
            cwd: None,
            tool_name: None,
            tool_input: None,
            tool_response: None,
            prompt: None,
            error: None,
            last_assistant_message: None,
            source: None,
            reason: None,
            trigger: None,
        }
    }

    #[test]
    fn escalation_threshold() {
        assert!(ESCALATION_COMPETENCE_THRESHOLD > 0.0);
        assert!(ESCALATION_COMPETENCE_THRESHOLD < 1.0);
    }

    #[test]
    fn overconfidence_forces_revise() {
        let mut ctx = LoopContext::new(make_input());
        ctx.overconfidence_score = 0.5; // above 0.3 threshold
        let stage = DecideStage;
        stage.run(&mut ctx);
        assert_eq!(ctx.decision, Decision::Revise);
    }

    #[test]
    fn plasticity_lost_forces_escalate() {
        let mut ctx = LoopContext::new(make_input());
        ctx.plasticity_lost = true;
        let stage = DecideStage;
        let result = stage.run(&mut ctx);
        assert_eq!(ctx.decision, Decision::Escalate);
        assert!(matches!(result, StageResult::Escalate(_)));
    }

    #[test]
    fn normal_operation_continues() {
        let mut ctx = LoopContext::new(make_input());
        // defaults: overconfidence_score=0.0, plasticity_lost=false
        let stage = DecideStage;
        stage.run(&mut ctx);
        assert_eq!(ctx.decision, Decision::Continue);
    }

    #[test]
    fn verification_errors_force_revise() {
        let patterns = [
            "test_failures: 3 tests failed",
            "tool_mismatch: expected Write, got Read",
            "syntax_error: unexpected token",
            "tool_error: command failed",
            "response_contains: error pattern detected",
        ];
        for pattern in &patterns {
            let mut ctx = LoopContext::new(make_input());
            ctx.verification_results = vec![pattern.to_string()];
            let stage = DecideStage;
            stage.run(&mut ctx);
            assert_eq!(
                ctx.decision,
                Decision::Revise,
                "pattern '{}' should force Revise",
                pattern
            );
        }
    }
}
