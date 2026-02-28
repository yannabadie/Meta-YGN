use metaygn_shared::budget_tracker::SessionBudget;

#[test]
fn new_budget_starts_at_zero_consumed() {
    let budget = SessionBudget::new(10_000, 0.10);
    assert_eq!(budget.consumed_tokens(), 0);
    assert!((budget.consumed_cost_usd() - 0.0).abs() < f64::EPSILON);
    assert!(!budget.is_over_budget());
}

#[test]
fn consume_tokens_updates_state() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(500, 0.005);
    assert_eq!(budget.consumed_tokens(), 500);
    assert!((budget.consumed_cost_usd() - 0.005).abs() < 1e-9);
}

#[test]
fn over_budget_detected() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(10_001, 0.05);
    assert!(budget.is_over_budget());

    // Also test cost-based over-budget
    let mut budget2 = SessionBudget::new(10_000, 0.10);
    budget2.consume(5_000, 0.11);
    assert!(budget2.is_over_budget());
}

#[test]
fn utilization_percentage() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(8_000, 0.08);
    let util = budget.utilization();
    assert!(
        (util - 0.80).abs() < 1e-9,
        "Expected utilization 0.80, got {util}"
    );
}

#[test]
fn warning_at_80_percent() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(8_500, 0.085);
    assert!(
        budget.should_warn(),
        "Expected warning at 85% utilization"
    );
}

#[test]
fn no_warning_below_80_percent() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(5_000, 0.05);
    assert!(
        !budget.should_warn(),
        "Expected no warning at 50% utilization"
    );
}

#[test]
fn budget_summary_string() {
    let mut budget = SessionBudget::new(10_000, 0.10);
    budget.consume(3_000, 0.03);
    let summary = budget.summary();
    assert!(
        summary.contains("3000tok"),
        "Expected consumed tokens in summary, got: {summary}"
    );
    assert!(
        summary.contains("$0.03"),
        "Expected consumed cost in summary, got: {summary}"
    );
    assert!(
        summary.contains("10000tok"),
        "Expected max tokens in summary, got: {summary}"
    );
    assert!(
        summary.contains("$0.10"),
        "Expected max cost in summary, got: {summary}"
    );
    assert!(
        summary.contains("30%"),
        "Expected percentage in summary, got: {summary}"
    );
    assert!(
        summary.starts_with("[budget:"),
        "Expected summary to start with '[budget:', got: {summary}"
    );
}
