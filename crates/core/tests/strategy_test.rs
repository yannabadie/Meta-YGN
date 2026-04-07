use metaygn_core::context::LoopContext;
use metaygn_core::stages::strategy::StrategyStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::{RiskLevel, Strategy, TaskType};

/// Helper: build a default `HookInput`.
fn default_input() -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        ..Default::default()
    }
}

/// Helper: run the strategy stage with given risk, difficulty, and task type.
/// Returns the selected strategy.
fn select(risk: RiskLevel, difficulty: f32, task_type: Option<TaskType>) -> Strategy {
    let stage = StrategyStage;
    let mut ctx = LoopContext::new(default_input());
    ctx.risk = risk;
    ctx.difficulty = difficulty;
    ctx.task_type = task_type;
    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue, "strategy stage should always return Continue");
    ctx.strategy
}

// ─── Stage metadata ───────────────────────────────────────────────────────────

#[test]
fn stage_name_is_strategy() {
    let stage = StrategyStage;
    assert_eq!(stage.name(), "strategy");
}

#[test]
fn stage_always_returns_continue() {
    let stage = StrategyStage;
    let mut ctx = LoopContext::new(default_input());
    assert_eq!(stage.run(&mut ctx), StageResult::Continue);
}

// ─── Default context before strategy runs ─────────────────────────────────────

#[test]
fn default_strategy_is_step_by_step() {
    let ctx = LoopContext::new(default_input());
    assert_eq!(ctx.strategy, Strategy::StepByStep, "default strategy should be StepByStep");
}

// ─── Task-type overrides (highest priority) ──────────────────────────────────

#[test]
fn security_always_adversarial_low_risk_low_difficulty() {
    assert_eq!(select(RiskLevel::Low, 0.1, Some(TaskType::Security)), Strategy::Adversarial);
}

#[test]
fn security_always_adversarial_high_risk_high_difficulty() {
    assert_eq!(select(RiskLevel::High, 0.9, Some(TaskType::Security)), Strategy::Adversarial);
}

#[test]
fn security_always_adversarial_medium_risk_medium_difficulty() {
    assert_eq!(select(RiskLevel::Medium, 0.5, Some(TaskType::Security)), Strategy::Adversarial);
}

#[test]
fn research_always_tree_explore_low_risk() {
    assert_eq!(select(RiskLevel::Low, 0.1, Some(TaskType::Research)), Strategy::TreeExplore);
}

#[test]
fn research_always_tree_explore_high_risk() {
    assert_eq!(select(RiskLevel::High, 0.9, Some(TaskType::Research)), Strategy::TreeExplore);
}

#[test]
fn research_always_tree_explore_medium_risk() {
    assert_eq!(select(RiskLevel::Medium, 0.5, Some(TaskType::Research)), Strategy::TreeExplore);
}

// ─── Non-override task types fall through to matrix ──────────────────────────

#[test]
fn bugfix_does_not_override_uses_matrix() {
    // Low risk, low difficulty => Rapid (same as None task type)
    assert_eq!(select(RiskLevel::Low, 0.1, Some(TaskType::Bugfix)), Strategy::Rapid);
}

#[test]
fn feature_does_not_override_uses_matrix() {
    assert_eq!(select(RiskLevel::Medium, 0.5, Some(TaskType::Feature)), Strategy::TreeExplore);
}

#[test]
fn refactor_does_not_override_uses_matrix() {
    assert_eq!(select(RiskLevel::High, 0.1, Some(TaskType::Refactor)), Strategy::VerifyFirst);
}

#[test]
fn architecture_does_not_override_uses_matrix() {
    assert_eq!(select(RiskLevel::Low, 0.9, Some(TaskType::Architecture)), Strategy::DivideConquer);
}

#[test]
fn release_does_not_override_uses_matrix() {
    assert_eq!(select(RiskLevel::Medium, 0.8, Some(TaskType::Release)), Strategy::Iterative);
}

// ─── Risk x Difficulty matrix: Low risk ──────────────────────────────────────

#[test]
fn low_risk_low_difficulty_rapid() {
    assert_eq!(select(RiskLevel::Low, 0.0, None), Strategy::Rapid);
}

#[test]
fn low_risk_low_difficulty_boundary() {
    // difficulty 0.29 is still < 0.3, so Low band
    assert_eq!(select(RiskLevel::Low, 0.29, None), Strategy::Rapid);
}

#[test]
fn low_risk_medium_difficulty_step_by_step() {
    assert_eq!(select(RiskLevel::Low, 0.3, None), Strategy::StepByStep);
}

#[test]
fn low_risk_medium_difficulty_mid() {
    assert_eq!(select(RiskLevel::Low, 0.5, None), Strategy::StepByStep);
}

#[test]
fn low_risk_medium_difficulty_upper_boundary() {
    // difficulty 0.69 is still < 0.7, so Medium band
    assert_eq!(select(RiskLevel::Low, 0.69, None), Strategy::StepByStep);
}

#[test]
fn low_risk_high_difficulty_divide_conquer() {
    assert_eq!(select(RiskLevel::Low, 0.7, None), Strategy::DivideConquer);
}

#[test]
fn low_risk_high_difficulty_max() {
    assert_eq!(select(RiskLevel::Low, 1.0, None), Strategy::DivideConquer);
}

// ─── Risk x Difficulty matrix: Medium risk ───────────────────────────────────

#[test]
fn medium_risk_low_difficulty_step_by_step() {
    assert_eq!(select(RiskLevel::Medium, 0.0, None), Strategy::StepByStep);
}

#[test]
fn medium_risk_low_difficulty_boundary() {
    assert_eq!(select(RiskLevel::Medium, 0.29, None), Strategy::StepByStep);
}

#[test]
fn medium_risk_medium_difficulty_tree_explore() {
    assert_eq!(select(RiskLevel::Medium, 0.5, None), Strategy::TreeExplore);
}

#[test]
fn medium_risk_medium_difficulty_lower_boundary() {
    assert_eq!(select(RiskLevel::Medium, 0.3, None), Strategy::TreeExplore);
}

#[test]
fn medium_risk_high_difficulty_iterative() {
    assert_eq!(select(RiskLevel::Medium, 0.7, None), Strategy::Iterative);
}

#[test]
fn medium_risk_high_difficulty_max() {
    assert_eq!(select(RiskLevel::Medium, 1.0, None), Strategy::Iterative);
}

// ─── Risk x Difficulty matrix: High risk ─────────────────────────────────────

#[test]
fn high_risk_low_difficulty_verify_first() {
    assert_eq!(select(RiskLevel::High, 0.0, None), Strategy::VerifyFirst);
}

#[test]
fn high_risk_low_difficulty_boundary() {
    assert_eq!(select(RiskLevel::High, 0.29, None), Strategy::VerifyFirst);
}

#[test]
fn high_risk_medium_difficulty_adversarial() {
    assert_eq!(select(RiskLevel::High, 0.5, None), Strategy::Adversarial);
}

#[test]
fn high_risk_medium_difficulty_lower_boundary() {
    assert_eq!(select(RiskLevel::High, 0.3, None), Strategy::Adversarial);
}

#[test]
fn high_risk_high_difficulty_verify_first() {
    assert_eq!(select(RiskLevel::High, 0.7, None), Strategy::VerifyFirst);
}

#[test]
fn high_risk_high_difficulty_max() {
    assert_eq!(select(RiskLevel::High, 1.0, None), Strategy::VerifyFirst);
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn none_task_type_uses_matrix() {
    // No task type override, falls through to matrix
    assert_eq!(select(RiskLevel::Low, 0.1, None), Strategy::Rapid);
}

#[test]
fn strategy_stage_overwrites_default() {
    let stage = StrategyStage;
    let mut ctx = LoopContext::new(default_input());
    // Default is StepByStep, but with Low risk and low difficulty => Rapid
    ctx.risk = RiskLevel::Low;
    ctx.difficulty = 0.1;
    ctx.task_type = None;
    stage.run(&mut ctx);
    assert_eq!(ctx.strategy, Strategy::Rapid, "strategy stage should overwrite the default");
}

#[test]
fn difficulty_exactly_at_boundary_03() {
    // 0.3 is >= 0.3 and < 0.7, so Medium band
    assert_eq!(select(RiskLevel::Low, 0.3, None), Strategy::StepByStep);
}

#[test]
fn difficulty_exactly_at_boundary_07() {
    // 0.7 is >= 0.7, so High band
    assert_eq!(select(RiskLevel::Low, 0.7, None), Strategy::DivideConquer);
}

#[test]
fn negative_difficulty_treated_as_low() {
    // Negative value is < 0.3, so Low band
    assert_eq!(select(RiskLevel::Low, -0.5, None), Strategy::Rapid);
}

#[test]
fn difficulty_above_1_treated_as_high() {
    // Value > 1.0 is >= 0.7, so High band
    assert_eq!(select(RiskLevel::Medium, 1.5, None), Strategy::Iterative);
}
