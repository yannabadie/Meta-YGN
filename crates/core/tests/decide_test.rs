use metaygn_core::context::LoopContext;
use metaygn_core::stages::decide::DecideStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::{Decision, MetacognitiveVector, RiskLevel};

/// Helper: build a default `HookInput` for testing.
fn make_input() -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        ..Default::default()
    }
}

/// Helper: run the decide stage, return (Decision, StageResult).
fn decide(ctx: &mut LoopContext) -> (Decision, StageResult) {
    let stage = DecideStage;
    let result = stage.run(ctx);
    (ctx.decision, result)
}

// ─── Stage metadata ──────────────────────────────────────────────────────────

#[test]
fn stage_name_is_decide() {
    let stage = DecideStage;
    assert_eq!(stage.name(), "decide");
}

// ─── Continue / Allow cases ──────────────────────────────────────────────────

#[test]
fn default_context_continues() {
    // Default LoopContext: risk=Low, competence=0.7, overconfidence=0.0,
    // plasticity_lost=false, quality=0.4 (above 0.3), no verification errors.
    let mut ctx = LoopContext::new(make_input());
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn low_risk_low_competence_still_continues() {
    // Low risk + low competence does NOT escalate (only High risk triggers it).
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::Low;
    ctx.competence = 0.1;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn medium_risk_low_competence_still_continues() {
    // Medium risk + low competence does NOT escalate (only High risk triggers it).
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::Medium;
    ctx.competence = 0.1;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn high_risk_high_competence_continues() {
    // High risk but competence >= 0.4 threshold does NOT escalate.
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.5;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn high_risk_at_exact_threshold_continues() {
    // Competence == 0.4 is NOT < 0.4, so should continue.
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.4;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn clean_verification_results_continue() {
    // Verification results that don't match error prefixes are ignored.
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec![
        "all_passed".to_string(),
        "check_ok: no issues".to_string(),
        "lint_clean".to_string(),
    ];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

// ─── Escalate cases ─────────────────────────────────────────────────────────

#[test]
fn high_risk_low_competence_escalates() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.3; // below 0.4 threshold
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn high_risk_zero_competence_escalates() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.0;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn high_risk_just_below_threshold_escalates() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.39;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn escalation_reason_mentions_risk_and_competence() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.2;
    let (_, result) = decide(&mut ctx);
    if let StageResult::Escalate(reason) = result {
        assert!(reason.contains("high risk"), "reason should mention high risk: {}", reason);
        assert!(reason.contains("competence"), "reason should mention competence: {}", reason);
    } else {
        panic!("expected StageResult::Escalate");
    }
}

#[test]
fn plasticity_lost_escalates() {
    let mut ctx = LoopContext::new(make_input());
    ctx.plasticity_lost = true;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn plasticity_lost_reason_mentions_plasticity() {
    let mut ctx = LoopContext::new(make_input());
    ctx.plasticity_lost = true;
    let (_, result) = decide(&mut ctx);
    if let StageResult::Escalate(reason) = result {
        assert!(
            reason.contains("plasticity"),
            "reason should mention plasticity: {}",
            reason
        );
    } else {
        panic!("expected StageResult::Escalate");
    }
}

// ─── Revise cases ────────────────────────────────────────────────────────────

#[test]
fn overconfidence_above_threshold_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.5;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn overconfidence_just_above_threshold_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.31;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn overconfidence_at_exact_threshold_does_not_revise() {
    // overconfidence_score > 0.3 triggers revise, so exactly 0.3 should NOT.
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.3;
    let (decision, _) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
}

#[test]
fn overconfidence_max_value_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 1.0;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn low_metacognitive_quality_revises() {
    // Set all metacog dimensions very low so overall_quality < 0.3.
    // quality = (conf + coh + grnd + (1 - complex) + prog) / 5
    // With all at 0.0 except complexity at 1.0: (0 + 0 + 0 + 0 + 0) / 5 = 0.0
    let mut ctx = LoopContext::new(make_input());
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.0,
        coherence: 0.0,
        grounding: 0.0,
        complexity: 1.0,
        progress: 0.0,
    };
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn quality_just_below_threshold_revises() {
    // Need quality < 0.3. Let's target ~0.29:
    // quality = (conf + coh + grnd + (1 - complex) + prog) / 5
    // (0.2 + 0.2 + 0.2 + (1.0 - 1.0) + 0.0) / 5 = 0.6 / 5 = 0.12 < 0.3
    let mut ctx = LoopContext::new(make_input());
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.2,
        coherence: 0.2,
        grounding: 0.2,
        complexity: 1.0,
        progress: 0.0,
    };
    assert!(ctx.metacog_vector.overall_quality() < 0.3);
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn quality_at_exact_threshold_does_not_revise() {
    // quality == 0.3 is NOT < 0.3, so should continue.
    // (0.3 + 0.3 + 0.3 + (1.0 - 1.0) + 0.6) / 5 = 1.5 / 5 = 0.3
    let mut ctx = LoopContext::new(make_input());
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.3,
        coherence: 0.3,
        grounding: 0.3,
        complexity: 1.0,
        progress: 0.6,
    };
    let quality = ctx.metacog_vector.overall_quality();
    assert!(
        (quality - 0.3).abs() < 1e-10,
        "expected quality ~0.3, got {}",
        quality
    );
    let (decision, _) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
}

// ─── Verification error patterns ─────────────────────────────────────────────

#[test]
fn verification_tool_error_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec!["tool_error: command failed with exit code 1".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_response_contains_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec!["response_contains: error pattern detected".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_test_failures_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec!["test_failures: 3 tests failed".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_tool_mismatch_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec!["tool_mismatch: expected Write, got Read".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_syntax_error_revises() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec!["syntax_error: unexpected token at line 42".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_error_among_clean_results_revises() {
    // One error among several clean results is enough to trigger Revise.
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec![
        "all_passed".to_string(),
        "lint_ok".to_string(),
        "test_failures: 1 test failed".to_string(),
        "formatting_ok".to_string(),
    ];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn verification_unknown_prefix_does_not_revise() {
    // Prefixes not in the known error list should NOT trigger revise.
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec![
        "warning: something might be off".to_string(),
        "info: build completed".to_string(),
    ];
    let (decision, _) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
}

#[test]
fn empty_verification_results_does_not_revise() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = Vec::new();
    let (decision, _) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
}

// ─── Priority ordering ──────────────────────────────────────────────────────

#[test]
fn escalation_takes_priority_over_overconfidence() {
    // Both high risk + low competence AND overconfidence are set.
    // Escalation check comes first in the code.
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.2;
    ctx.overconfidence_score = 0.5;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn escalation_takes_priority_over_plasticity_lost() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::High;
    ctx.competence = 0.2;
    ctx.plasticity_lost = true;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
    // The reason should be about risk+competence, not plasticity.
    if let StageResult::Escalate(reason) = result {
        assert!(reason.contains("competence"), "first escalation path should win: {}", reason);
    }
}

#[test]
fn overconfidence_takes_priority_over_plasticity_lost() {
    // overconfidence check comes before plasticity_lost in the code.
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.5;
    ctx.plasticity_lost = true;
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn overconfidence_takes_priority_over_low_quality() {
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.5;
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.0,
        coherence: 0.0,
        grounding: 0.0,
        complexity: 1.0,
        progress: 0.0,
    };
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn overconfidence_takes_priority_over_verification_errors() {
    let mut ctx = LoopContext::new(make_input());
    ctx.overconfidence_score = 0.5;
    ctx.verification_results = vec!["tool_error: failed".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn plasticity_lost_takes_priority_over_low_quality() {
    let mut ctx = LoopContext::new(make_input());
    ctx.plasticity_lost = true;
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.0,
        coherence: 0.0,
        grounding: 0.0,
        complexity: 1.0,
        progress: 0.0,
    };
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Escalate);
    assert!(matches!(result, StageResult::Escalate(_)));
}

#[test]
fn low_quality_takes_priority_over_verification_errors() {
    // Both low quality AND verification errors. Low quality check comes first.
    let mut ctx = LoopContext::new(make_input());
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.0,
        coherence: 0.0,
        grounding: 0.0,
        complexity: 1.0,
        progress: 0.0,
    };
    ctx.verification_results = vec!["tool_error: failed".to_string()];
    let (decision, result) = decide(&mut ctx);
    // Both paths lead to Revise; confirm the stage still results in Revise.
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn decision_field_is_set_by_stage() {
    // Verify that the decision field on the context is actually updated.
    let mut ctx = LoopContext::new(make_input());
    assert_eq!(ctx.decision, Decision::Continue, "default decision before stage runs");
    ctx.plasticity_lost = true;
    decide(&mut ctx);
    assert_eq!(ctx.decision, Decision::Escalate, "decision should be updated after stage runs");
}

#[test]
fn multiple_verification_errors_still_revise() {
    let mut ctx = LoopContext::new(make_input());
    ctx.verification_results = vec![
        "tool_error: a".to_string(),
        "test_failures: b".to_string(),
        "syntax_error: c".to_string(),
    ];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Revise);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn high_quality_with_clean_verification_continues() {
    let mut ctx = LoopContext::new(make_input());
    ctx.metacog_vector = MetacognitiveVector {
        confidence: 0.9,
        coherence: 0.9,
        grounding: 0.9,
        complexity: 0.1,
        progress: 0.9,
    };
    ctx.verification_results = vec!["all_passed".to_string()];
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn all_default_values_produce_continue() {
    // LoopContext::new defaults should lead to Continue (quality 0.4 > 0.3).
    let mut ctx = LoopContext::new(make_input());
    let quality = ctx.metacog_vector.overall_quality();
    assert!(
        quality >= 0.3,
        "default quality ({}) should be >= 0.3",
        quality
    );
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}

#[test]
fn risk_level_low_with_all_other_triggers_false_continues() {
    let mut ctx = LoopContext::new(make_input());
    ctx.risk = RiskLevel::Low;
    ctx.competence = 0.7;
    ctx.overconfidence_score = 0.0;
    ctx.plasticity_lost = false;
    ctx.verification_results = Vec::new();
    let (decision, result) = decide(&mut ctx);
    assert_eq!(decision, Decision::Continue);
    assert_eq!(result, StageResult::Continue);
}
