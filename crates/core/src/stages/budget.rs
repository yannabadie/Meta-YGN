use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::RiskLevel;

/// Stage 5: Set token budget based on assessed difficulty and risk.
pub struct BudgetStage;

impl Stage for BudgetStage {
    fn name(&self) -> &'static str {
        "budget"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        // Token budget scales with difficulty.
        ctx.budget.max_tokens = tokens_for_difficulty(ctx.difficulty);

        // Risk tolerance mirrors the assessed risk.
        ctx.budget.risk_tolerance = ctx.risk;

        // Latency budget: tighter for low-risk, more generous for high-risk.
        ctx.budget.max_latency_ms = match ctx.risk {
            RiskLevel::Low => 10_000,
            RiskLevel::Medium => 30_000,
            RiskLevel::High => 60_000,
        };

        // Cost cap scales with token budget.
        ctx.budget.max_cost_usd = ctx.budget.max_tokens as f64 * 0.00002;

        tracing::debug!(
            stage = self.name(),
            max_tokens = ctx.budget.max_tokens,
            max_latency_ms = ctx.budget.max_latency_ms,
            "set budget"
        );

        StageResult::Continue
    }
}

/// Map difficulty to a token budget.
///
/// - Low difficulty  (< 0.3):  1 000 tokens
/// - Medium difficulty (0.3..0.7): 5 000 tokens
/// - High difficulty  (>= 0.7): 20 000 tokens
fn tokens_for_difficulty(difficulty: f32) -> u64 {
    if difficulty < 0.3 {
        1_000
    } else if difficulty < 0.7 {
        5_000
    } else {
        20_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_difficulty_budget() {
        assert_eq!(tokens_for_difficulty(0.1), 1_000);
    }

    #[test]
    fn medium_difficulty_budget() {
        assert_eq!(tokens_for_difficulty(0.5), 5_000);
    }

    #[test]
    fn high_difficulty_budget() {
        assert_eq!(tokens_for_difficulty(0.9), 20_000);
    }
}
