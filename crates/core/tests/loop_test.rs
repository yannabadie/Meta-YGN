use metaygn_core::context::LoopContext;
use metaygn_core::runner::ControlLoop;
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::{Decision, RiskLevel, Strategy, TaskType};

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
fn control_loop_runs_all_stages() {
    let cl = ControlLoop::new();
    let input = make_input(Some("hello world"));
    let mut ctx = LoopContext::new(input);

    let decision = cl.run(&mut ctx);

    // A benign input should result in Continue.
    assert_eq!(decision, Decision::Continue);

    // Lessons should have been collected by the learn stage.
    assert!(
        !ctx.lessons.is_empty(),
        "learn stage should populate lessons"
    );

    // The metacog vector should have been updated by calibrate.
    assert!(
        ctx.metacog_vector.progress > 0.0,
        "progress should be non-zero after calibration"
    );
}

#[test]
fn control_loop_classifies_bugfix() {
    let cl = ControlLoop::new();
    let input = make_input(Some("fix the login bug where users cannot sign in"));
    let mut ctx = LoopContext::new(input);

    cl.run(&mut ctx);

    assert_eq!(
        ctx.task_type,
        Some(TaskType::Bugfix),
        "prompt with 'fix' and 'bug' should classify as Bugfix"
    );
}

#[test]
fn control_loop_high_risk_strategy() {
    let cl = ControlLoop::new();

    // A prompt that triggers high risk: contains "bash" tool.
    let mut input = make_input(Some("deploy the application to production"));
    input.tool_name = Some("bash".to_string());
    let mut ctx = LoopContext::new(input);

    cl.run(&mut ctx);

    assert_eq!(ctx.risk, RiskLevel::High, "bash tool should be high risk");

    // High risk with non-security, non-research task should use VerifyFirst
    // (unless task_type overrides).
    // Since "deploy" classifies as Release (not Security/Research),
    // the strategy matrix at High risk should yield VerifyFirst or Adversarial.
    assert!(
        ctx.strategy == Strategy::VerifyFirst || ctx.strategy == Strategy::Adversarial,
        "high risk should use VerifyFirst or Adversarial, got {:?}",
        ctx.strategy
    );
}

#[test]
fn control_loop_escalates_when_stuck() {
    let cl = ControlLoop::new();

    // Craft an input that produces high risk + low competence:
    // - "bash" tool => high risk
    // - "security vulnerability" => TaskType::Security => competence 0.4
    // - "quantum" keyword => competence penalty 0.1 => 0.3 (below 0.4 threshold)
    let mut input = make_input(Some(
        "check for quantum security vulnerability in the kernel driver",
    ));
    input.tool_name = Some("bash".to_string());
    let mut ctx = LoopContext::new(input);

    let decision = cl.run(&mut ctx);

    assert_eq!(
        decision,
        Decision::Escalate,
        "high risk + low competence should escalate"
    );

    // The escalation reason should be recorded in lessons.
    let has_escalation_lesson = ctx.lessons.iter().any(|l| l.contains("escalat"));
    assert!(
        has_escalation_lesson,
        "lessons should mention escalation, got: {:?}",
        ctx.lessons
    );
}

#[test]
fn control_loop_sets_budget_by_difficulty() {
    let cl = ControlLoop::new();

    // Simple prompt => low difficulty => small budget.
    let input = make_input(Some("hello"));
    let mut ctx = LoopContext::new(input);
    cl.run(&mut ctx);
    assert!(
        ctx.budget.max_tokens <= 5_000,
        "short prompt should get small budget, got {}",
        ctx.budget.max_tokens
    );

    // Complex prompt => higher difficulty => bigger budget.
    let input = make_input(Some(
        "implement a distributed concurrent system with async parallel \
         processing and performance optimization for complex recursive \
         cryptographic operations at scale with backward compatibility",
    ));
    let mut ctx = LoopContext::new(input);
    cl.run(&mut ctx);
    assert!(
        ctx.budget.max_tokens >= 5_000,
        "complex prompt should get larger budget, got {}",
        ctx.budget.max_tokens
    );
}

#[test]
fn control_loop_detects_tool_need() {
    let cl = ControlLoop::new();

    // Without tool_name => not necessary.
    let input = make_input(Some("explain how async works"));
    let mut ctx = LoopContext::new(input);
    cl.run(&mut ctx);
    assert!(!ctx.tool_necessary, "no tool_name means tool not necessary");

    // With tool_name => necessary.
    let mut input = make_input(Some("read the file"));
    input.tool_name = Some("Read".to_string());
    let mut ctx = LoopContext::new(input);
    cl.run(&mut ctx);
    assert!(
        ctx.tool_necessary,
        "tool_name present means tool is necessary"
    );
}

#[test]
fn control_loop_stage_count_is_twelve() {
    let cl = ControlLoop::new();
    assert_eq!(cl.stage_count(), 12);
}

#[test]
fn act_stage_records_intended_action() {
    let cl = ControlLoop::new();

    let mut input = make_input(Some("read the config file"));
    input.tool_name = Some("Read".to_string());
    input.tool_input = Some(serde_json::json!({
        "file_path": "/etc/config.toml"
    }));
    let mut ctx = LoopContext::new(input);

    cl.run(&mut ctx);

    let action = ctx
        .intended_action
        .as_ref()
        .expect("act stage should record an intended action");
    assert_eq!(action.tool, "Read");
    assert_eq!(action.target, "/etc/config.toml");
    assert!(
        action.purpose.contains("strategy"),
        "purpose should mention strategy, got: {}",
        action.purpose
    );
}

#[test]
fn compact_stage_clusters_lessons() {
    let cl = ControlLoop::new();
    let input = make_input(Some("hello world"));
    let mut ctx = LoopContext::new(input);

    // Pre-populate lessons with similar entries that share >50% word overlap.
    // "lesson A" / "lesson B" etc. only have one non-trivial word ("lesson"),
    // so they all cluster together under the new semantic clustering logic.
    ctx.lessons = vec![
        "always check error handling before deploying".into(),
        "always check error handling before releasing".into(), // similar
        "database migrations require careful backup planning".into(),
        "always check error handling before shipping".into(), // similar
        "unit tests should cover edge cases thoroughly".into(),
    ];

    cl.run(&mut ctx);

    // 1. The compact summary should exist
    let has_compact_summary = ctx.lessons.iter().any(|l| l.starts_with("[compact]"));
    assert!(
        has_compact_summary,
        "compact stage should add a summary, got: {:?}",
        ctx.lessons
    );

    // 2. The three similar "always check error handling" lessons should be
    //    merged into one cluster with a "(x3)" suffix.
    let merged_count = ctx
        .lessons
        .iter()
        .filter(|l| l.contains("error handling") && l.contains("(x3)"))
        .count();
    assert_eq!(
        merged_count, 1,
        "similar lessons should merge into one cluster with (x3), got: {:?}",
        ctx.lessons
    );
}
