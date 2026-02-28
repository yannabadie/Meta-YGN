use super::{Stage, StageResult};
use crate::context::LoopContext;
use metaygn_shared::state::{RiskLevel, Strategy, TaskType};

/// Stage 6: Select a reasoning strategy based on risk, difficulty, and task type.
pub struct StrategyStage;

impl Stage for StrategyStage {
    fn name(&self) -> &'static str {
        "strategy"
    }

    fn run(&self, ctx: &mut LoopContext) -> StageResult {
        ctx.strategy = select_strategy(ctx.risk, ctx.difficulty, ctx.task_type);

        tracing::debug!(
            stage = self.name(),
            strategy = ?ctx.strategy,
            "selected strategy"
        );

        StageResult::Continue
    }
}

/// Strategy selection matrix.
///
/// | Risk \ Difficulty | Low (< 0.3) | Medium (0.3..0.7) | High (>= 0.7) |
/// |---|---|---|---|
/// | **Low**  | Rapid | StepByStep | DivideConquer |
/// | **Medium** | StepByStep | TreeExplore | Iterative |
/// | **High** | VerifyFirst | Adversarial | VerifyFirst |
///
/// Task-type overrides:
/// - Security always gets `Adversarial`.
/// - Research always gets `TreeExplore`.
fn select_strategy(risk: RiskLevel, difficulty: f32, task_type: Option<TaskType>) -> Strategy {
    // Task-type overrides take precedence.
    if let Some(tt) = task_type {
        match tt {
            TaskType::Security => return Strategy::Adversarial,
            TaskType::Research => return Strategy::TreeExplore,
            _ => {}
        }
    }

    match (risk, difficulty_band(difficulty)) {
        (RiskLevel::Low, DifficultyBand::Low) => Strategy::Rapid,
        (RiskLevel::Low, DifficultyBand::Medium) => Strategy::StepByStep,
        (RiskLevel::Low, DifficultyBand::High) => Strategy::DivideConquer,
        (RiskLevel::Medium, DifficultyBand::Low) => Strategy::StepByStep,
        (RiskLevel::Medium, DifficultyBand::Medium) => Strategy::TreeExplore,
        (RiskLevel::Medium, DifficultyBand::High) => Strategy::Iterative,
        (RiskLevel::High, DifficultyBand::Low) => Strategy::VerifyFirst,
        (RiskLevel::High, DifficultyBand::Medium) => Strategy::Adversarial,
        (RiskLevel::High, DifficultyBand::High) => Strategy::VerifyFirst,
    }
}

#[derive(Debug, Clone, Copy)]
enum DifficultyBand {
    Low,
    Medium,
    High,
}

fn difficulty_band(d: f32) -> DifficultyBand {
    if d < 0.3 {
        DifficultyBand::Low
    } else if d < 0.7 {
        DifficultyBand::Medium
    } else {
        DifficultyBand::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_risk_gets_verify_first() {
        let s = select_strategy(RiskLevel::High, 0.1, Some(TaskType::Feature));
        assert_eq!(s, Strategy::VerifyFirst);
    }

    #[test]
    fn security_always_adversarial() {
        let s = select_strategy(RiskLevel::Low, 0.1, Some(TaskType::Security));
        assert_eq!(s, Strategy::Adversarial);
    }

    #[test]
    fn low_risk_low_difficulty_rapid() {
        let s = select_strategy(RiskLevel::Low, 0.1, Some(TaskType::Bugfix));
        assert_eq!(s, Strategy::Rapid);
    }
}
