use metaygn_core::context::{IntendedAction, LoopContext};
use metaygn_core::stages::verify::VerifyStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};

/// Helper: build a minimal `HookInput` for verify-stage tests.
fn make_input() -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PostToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
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
fn verify_detects_intended_action_mismatch() {
    let stage = VerifyStage;

    let mut input = make_input();
    input.tool_name = Some("Bash".to_string());
    input.tool_response = Some("ok".to_string());

    let mut ctx = LoopContext::new(input);
    ctx.intended_action = Some(IntendedAction {
        tool: "Write".to_string(),
        target: "/tmp/file.txt".to_string(),
        purpose: "create config".to_string(),
    });

    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue);

    let has_mismatch = ctx
        .verification_results
        .iter()
        .any(|r| r.contains("tool_mismatch"));
    assert!(
        has_mismatch,
        "should detect tool mismatch when intended='Write' but executed='Bash', got: {:?}",
        ctx.verification_results
    );
}

#[test]
fn verify_parses_test_failures() {
    let stage = VerifyStage;

    let mut input = make_input();
    input.tool_name = Some("Bash".to_string());
    input.tool_response = Some("test result: ok. 5 passed; 2 failed".to_string());

    let mut ctx = LoopContext::new(input);

    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue);

    let has_failures = ctx
        .verification_results
        .iter()
        .any(|r| r.contains("test_failures: 2"));
    assert!(
        has_failures,
        "should parse '2 failed' from test output, got: {:?}",
        ctx.verification_results
    );
}

#[test]
fn verify_keyword_scan_still_works() {
    let stage = VerifyStage;

    let mut input = make_input();
    input.tool_response = Some("Something went wrong: error in module X".to_string());

    let mut ctx = LoopContext::new(input);

    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue);

    let has_error = ctx
        .verification_results
        .iter()
        .any(|r| r.contains("response_contains: error"));
    assert!(
        has_error,
        "keyword scan should detect 'error' in response, got: {:?}",
        ctx.verification_results
    );
}

#[test]
fn verify_no_false_positive_on_success() {
    let stage = VerifyStage;

    let mut input = make_input();
    input.tool_name = Some("Bash".to_string());
    input.tool_response = Some("All 42 tests passed".to_string());

    let mut ctx = LoopContext::new(input);

    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue);

    let has_test_failure = ctx
        .verification_results
        .iter()
        .any(|r| r.contains("test_failures"));
    assert!(
        !has_test_failure,
        "should not report test_failures when all tests passed, got: {:?}",
        ctx.verification_results
    );

    let has_tool_error = ctx
        .verification_results
        .iter()
        .any(|r| r.contains("tool_error"));
    assert!(
        !has_tool_error,
        "should not report tool_error on success, got: {:?}",
        ctx.verification_results
    );
}
