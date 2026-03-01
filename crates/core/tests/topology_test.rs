use metaygn_core::context::LoopContext;
use metaygn_core::runner::ControlLoop;
use metaygn_core::topology::{ALL_STAGES, TopologyPlanner};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::{Decision, RiskLevel, TaskType, Topology};

/// Helper: build a minimal `HookInput` with an optional prompt.
fn make_input(prompt: Option<&str>) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        tool_name: None,
        tool_input: None,
        tool_response: None,
        prompt: prompt.map(|s| s.to_string()),
        error: None,
        last_assistant_message: None,
        source: None,
        reason: None,
        trigger: None,
    }
}

#[test]
fn trivial_task_gets_4_stages() {
    let plan = TopologyPlanner::plan(RiskLevel::Low, 0.1, TaskType::Bugfix);
    assert_eq!(plan.topology, Topology::Single);
    assert_eq!(plan.stages.len(), 4);
    assert_eq!(plan.stages, vec!["classify", "assess", "act", "decide"]);
}

#[test]
fn medium_risk_gets_full_pipeline() {
    let plan = TopologyPlanner::plan(RiskLevel::Medium, 0.5, TaskType::Feature);
    assert_eq!(plan.topology, Topology::Vertical);
    assert_eq!(plan.stages.len(), 12);
    assert_eq!(plan.stages, ALL_STAGES.to_vec());
}

#[test]
fn high_risk_gets_double_verify() {
    let plan = TopologyPlanner::plan(RiskLevel::High, 0.8, TaskType::Architecture);
    assert_eq!(plan.topology, Topology::Horizontal);
    assert_eq!(
        plan.stages.len(),
        14,
        "12 base + 2 extra (verify + calibrate)"
    );
    // The last two stages should be the extra verify + calibrate pass.
    assert_eq!(plan.stages[12], "verify");
    assert_eq!(plan.stages[13], "calibrate");
}

#[test]
fn security_always_horizontal() {
    // Even with Low risk, security tasks should get Horizontal topology.
    let plan_low = TopologyPlanner::plan(RiskLevel::Low, 0.1, TaskType::Security);
    assert_eq!(plan_low.topology, Topology::Horizontal);
    assert_eq!(plan_low.stages.len(), 14);

    let plan_medium = TopologyPlanner::plan(RiskLevel::Medium, 0.5, TaskType::Security);
    assert_eq!(plan_medium.topology, Topology::Horizontal);

    let plan_high = TopologyPlanner::plan(RiskLevel::High, 0.9, TaskType::Security);
    assert_eq!(plan_high.topology, Topology::Horizontal);
}

#[test]
fn research_gets_slim_pipeline() {
    let plan = TopologyPlanner::plan(RiskLevel::Low, 0.5, TaskType::Research);
    assert_eq!(plan.topology, Topology::Vertical);
    assert_eq!(plan.stages.len(), 6);
    assert_eq!(
        plan.stages,
        vec![
            "classify",
            "assess",
            "competence",
            "strategy",
            "act",
            "learn"
        ]
    );
}

#[test]
fn run_plan_executes_subset() {
    let cl = ControlLoop::new();
    let input = make_input(Some("hello world"));
    let mut ctx = LoopContext::new(input);

    // Build a trivial plan (4 stages).
    let plan = TopologyPlanner::trivial_pipeline();
    assert_eq!(plan.stages.len(), 4);

    let decision = cl.run_plan(&mut ctx, &plan);

    // A benign input with a trivial plan should still reach a decision.
    assert_eq!(decision, Decision::Continue);

    // The learn stage was NOT in the trivial plan, so lessons should be empty.
    // (The decide stage does not add lessons -- only learn and escalation do.)
    let has_learn_lesson = ctx.lessons.iter().any(|l| l.contains("loop iteration"));
    assert!(
        !has_learn_lesson,
        "learn stage should NOT have run in trivial pipeline, but got lessons: {:?}",
        ctx.lessons
    );
}

#[test]
fn full_pipeline_has_12_stages() {
    let plan = TopologyPlanner::full_pipeline();
    assert_eq!(plan.topology, Topology::Vertical);
    assert_eq!(plan.stages.len(), 12);
    assert_eq!(plan.stages, ALL_STAGES.to_vec());
    assert!(plan.rationale.contains("12-stage"));
}
