use metaygn_core::context::LoopContext;
use metaygn_core::stages::budget::BudgetStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::budget_tracker::COST_PER_TOKEN_USD;
use metaygn_shared::state::RiskLevel;

/// Helper: build a default `HookInput`.
fn default_input() -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        ..Default::default()
    }
}

/// Helper: run the budget stage with given risk and difficulty.
/// Returns (max_tokens, max_latency_ms, max_cost_usd, risk_tolerance).
fn budget(risk: RiskLevel, difficulty: f32) -> (u64, u64, f64, RiskLevel) {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    ctx.risk = risk;
    ctx.difficulty = difficulty;
    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue, "budget stage should always return Continue");
    (
        ctx.budget.max_tokens,
        ctx.budget.max_latency_ms,
        ctx.budget.max_cost_usd,
        ctx.budget.risk_tolerance,
    )
}

// ─── Stage metadata ───────────────────────────────────────────────────────────

#[test]
fn stage_name_is_budget() {
    let stage = BudgetStage;
    assert_eq!(stage.name(), "budget");
}

#[test]
fn stage_always_returns_continue() {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    assert_eq!(stage.run(&mut ctx), StageResult::Continue);
}

// ─── Token budget based on difficulty ────────────────────────────────────────

#[test]
fn low_difficulty_gets_1000_tokens() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.0);
    assert_eq!(tokens, 1_000);
}

#[test]
fn low_difficulty_boundary_029() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.29);
    assert_eq!(tokens, 1_000);
}

#[test]
fn medium_difficulty_gets_5000_tokens() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.5);
    assert_eq!(tokens, 5_000);
}

#[test]
fn medium_difficulty_lower_boundary_03() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.3);
    assert_eq!(tokens, 5_000);
}

#[test]
fn medium_difficulty_upper_boundary_069() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.69);
    assert_eq!(tokens, 5_000);
}

#[test]
fn high_difficulty_gets_20000_tokens() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.9);
    assert_eq!(tokens, 20_000);
}

#[test]
fn high_difficulty_lower_boundary_07() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 0.7);
    assert_eq!(tokens, 20_000);
}

#[test]
fn high_difficulty_max_10() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 1.0);
    assert_eq!(tokens, 20_000);
}

// ─── Token budget is independent of risk ─────────────────────────────────────

#[test]
fn tokens_same_regardless_of_risk_low_difficulty() {
    let (t_low, _, _, _) = budget(RiskLevel::Low, 0.1);
    let (t_med, _, _, _) = budget(RiskLevel::Medium, 0.1);
    let (t_high, _, _, _) = budget(RiskLevel::High, 0.1);
    assert_eq!(t_low, t_med);
    assert_eq!(t_med, t_high);
    assert_eq!(t_low, 1_000);
}

#[test]
fn tokens_same_regardless_of_risk_high_difficulty() {
    let (t_low, _, _, _) = budget(RiskLevel::Low, 0.8);
    let (t_med, _, _, _) = budget(RiskLevel::Medium, 0.8);
    let (t_high, _, _, _) = budget(RiskLevel::High, 0.8);
    assert_eq!(t_low, t_med);
    assert_eq!(t_med, t_high);
    assert_eq!(t_low, 20_000);
}

// ─── Risk tolerance mirrors assessed risk ────────────────────────────────────

#[test]
fn risk_tolerance_mirrors_low_risk() {
    let (_, _, _, tolerance) = budget(RiskLevel::Low, 0.5);
    assert_eq!(tolerance, RiskLevel::Low);
}

#[test]
fn risk_tolerance_mirrors_medium_risk() {
    let (_, _, _, tolerance) = budget(RiskLevel::Medium, 0.5);
    assert_eq!(tolerance, RiskLevel::Medium);
}

#[test]
fn risk_tolerance_mirrors_high_risk() {
    let (_, _, _, tolerance) = budget(RiskLevel::High, 0.5);
    assert_eq!(tolerance, RiskLevel::High);
}

// ─── Latency budget based on risk ────────────────────────────────────────────

#[test]
fn low_risk_latency_10_seconds() {
    let (_, latency, _, _) = budget(RiskLevel::Low, 0.5);
    assert_eq!(latency, 10_000);
}

#[test]
fn medium_risk_latency_30_seconds() {
    let (_, latency, _, _) = budget(RiskLevel::Medium, 0.5);
    assert_eq!(latency, 30_000);
}

#[test]
fn high_risk_latency_60_seconds() {
    let (_, latency, _, _) = budget(RiskLevel::High, 0.5);
    assert_eq!(latency, 60_000);
}

// ─── Latency is independent of difficulty ────────────────────────────────────

#[test]
fn latency_same_regardless_of_difficulty_low_risk() {
    let (_, lat_low, _, _) = budget(RiskLevel::Low, 0.1);
    let (_, lat_high, _, _) = budget(RiskLevel::Low, 0.9);
    assert_eq!(lat_low, lat_high);
    assert_eq!(lat_low, 10_000);
}

#[test]
fn latency_same_regardless_of_difficulty_high_risk() {
    let (_, lat_low, _, _) = budget(RiskLevel::High, 0.1);
    let (_, lat_high, _, _) = budget(RiskLevel::High, 0.9);
    assert_eq!(lat_low, lat_high);
    assert_eq!(lat_low, 60_000);
}

// ─── Cost cap = max_tokens * COST_PER_TOKEN_USD ─────────────────────────────────────────

#[test]
fn cost_cap_low_difficulty() {
    let (_, _, cost, _) = budget(RiskLevel::Low, 0.1);
    let expected = 1_000.0 * COST_PER_TOKEN_USD;
    assert!((cost - expected).abs() < 1e-10, "cost should be {expected}, got {cost}");
}

#[test]
fn cost_cap_medium_difficulty() {
    let (_, _, cost, _) = budget(RiskLevel::Low, 0.5);
    let expected = 5_000.0 * COST_PER_TOKEN_USD;
    assert!((cost - expected).abs() < 1e-10, "cost should be {expected}, got {cost}");
}

#[test]
fn cost_cap_high_difficulty() {
    let (_, _, cost, _) = budget(RiskLevel::Low, 0.9);
    let expected = 20_000.0 * COST_PER_TOKEN_USD;
    assert!((cost - expected).abs() < 1e-10, "cost should be {expected}, got {cost}");
}

// ─── Default context before budget runs ──────────────────────────────────────

#[test]
fn default_budget_max_tokens() {
    let ctx = LoopContext::new(default_input());
    assert_eq!(ctx.budget.max_tokens, 5000, "default max_tokens should be 5000");
}

#[test]
fn default_budget_consumed_tokens() {
    let ctx = LoopContext::new(default_input());
    assert_eq!(ctx.budget.consumed_tokens, 0, "default consumed_tokens should be 0");
}

#[test]
fn default_budget_max_latency() {
    let ctx = LoopContext::new(default_input());
    assert_eq!(ctx.budget.max_latency_ms, 30_000, "default max_latency_ms should be 30000");
}

#[test]
fn default_budget_risk_tolerance() {
    let ctx = LoopContext::new(default_input());
    assert_eq!(ctx.budget.risk_tolerance, RiskLevel::Medium, "default risk_tolerance should be Medium");
}

// ─── Budget stage overwrites defaults ────────────────────────────────────────

#[test]
fn budget_stage_overwrites_default_tokens() {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    // Default difficulty is 0.5 (Medium band => 5000 tokens, same as default).
    // Set difficulty to 0.1 to force a different value.
    ctx.difficulty = 0.1;
    ctx.risk = RiskLevel::Low;
    stage.run(&mut ctx);
    assert_eq!(ctx.budget.max_tokens, 1_000, "budget stage should overwrite default tokens");
}

#[test]
fn budget_stage_overwrites_default_latency() {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    ctx.risk = RiskLevel::High;
    ctx.difficulty = 0.5;
    stage.run(&mut ctx);
    assert_eq!(ctx.budget.max_latency_ms, 60_000, "budget stage should overwrite default latency");
}

#[test]
fn budget_stage_overwrites_default_risk_tolerance() {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    ctx.risk = RiskLevel::High;
    ctx.difficulty = 0.5;
    stage.run(&mut ctx);
    assert_eq!(ctx.budget.risk_tolerance, RiskLevel::High, "budget stage should overwrite default risk_tolerance");
}

// ─── Consumed tokens are not touched by budget stage ─────────────────────────

#[test]
fn consumed_tokens_unchanged_by_budget_stage() {
    let stage = BudgetStage;
    let mut ctx = LoopContext::new(default_input());
    ctx.budget.consumed_tokens = 42;
    ctx.risk = RiskLevel::Low;
    ctx.difficulty = 0.5;
    stage.run(&mut ctx);
    assert_eq!(ctx.budget.consumed_tokens, 42, "budget stage should not modify consumed_tokens");
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn negative_difficulty_treated_as_low() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, -1.0);
    assert_eq!(tokens, 1_000, "negative difficulty should map to Low band (1000 tokens)");
}

#[test]
fn difficulty_above_1_treated_as_high() {
    let (tokens, _, _, _) = budget(RiskLevel::Low, 2.0);
    assert_eq!(tokens, 20_000, "difficulty > 1.0 should map to High band (20000 tokens)");
}

#[test]
fn all_fields_set_consistently_low_risk_low_difficulty() {
    let (tokens, latency, cost, tolerance) = budget(RiskLevel::Low, 0.1);
    assert_eq!(tokens, 1_000);
    assert_eq!(latency, 10_000);
    assert!((cost - 1_000.0 * COST_PER_TOKEN_USD).abs() < 1e-10);
    assert_eq!(tolerance, RiskLevel::Low);
}

#[test]
fn all_fields_set_consistently_high_risk_high_difficulty() {
    let (tokens, latency, cost, tolerance) = budget(RiskLevel::High, 0.9);
    assert_eq!(tokens, 20_000);
    assert_eq!(latency, 60_000);
    assert!((cost - 20_000.0 * COST_PER_TOKEN_USD).abs() < 1e-10);
    assert_eq!(tolerance, RiskLevel::High);
}
