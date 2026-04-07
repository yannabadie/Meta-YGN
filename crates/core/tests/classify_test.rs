use metaygn_core::context::LoopContext;
use metaygn_core::stages::classify::ClassifyStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};
use metaygn_shared::state::TaskType;

/// Helper: build a `HookInput` with only the prompt set.
fn input_with_prompt(prompt: &str) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        prompt: Some(prompt.to_string()),
        ..Default::default()
    }
}

/// Helper: build a `HookInput` with prompt and tool_name set.
fn input_with_prompt_and_tool(prompt: &str, tool_name: &str) -> HookInput {
    HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        prompt: Some(prompt.to_string()),
        tool_name: Some(tool_name.to_string()),
        ..Default::default()
    }
}

/// Helper: run the classify stage and return the assigned task type.
fn classify(input: HookInput) -> Option<TaskType> {
    let stage = ClassifyStage;
    let mut ctx = LoopContext::new(input);
    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue, "classify stage should always return Continue");
    ctx.task_type
}

// ─── Stage metadata ───────────────────────────────────────────────────────────

#[test]
fn stage_name_is_classify() {
    let stage = ClassifyStage;
    assert_eq!(stage.name(), "classify");
}

#[test]
fn stage_always_returns_continue() {
    let input = input_with_prompt("anything at all");
    let stage = ClassifyStage;
    let mut ctx = LoopContext::new(input);
    assert_eq!(stage.run(&mut ctx), StageResult::Continue);
}

// ─── Security (highest priority) ──────────────────────────────────────────────

#[test]
fn classifies_security_keyword_security() {
    assert_eq!(classify(input_with_prompt("audit the security settings")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_vulnerability() {
    assert_eq!(classify(input_with_prompt("found a vulnerability in auth")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_cve() {
    assert_eq!(classify(input_with_prompt("patch cve-2025-1234")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_auth() {
    assert_eq!(classify(input_with_prompt("review the auth flow")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_permission() {
    assert_eq!(classify(input_with_prompt("check file permission bits")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_secret() {
    assert_eq!(classify(input_with_prompt("rotate the api secret")), Some(TaskType::Security));
}

#[test]
fn classifies_security_keyword_credential() {
    assert_eq!(classify(input_with_prompt("update the credential store")), Some(TaskType::Security));
}

// ─── Bugfix ───────────────────────────────────────────────────────────────────

#[test]
fn classifies_bugfix_keyword_fix() {
    assert_eq!(classify(input_with_prompt("fix the login page")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_bug() {
    assert_eq!(classify(input_with_prompt("there is a bug in parsing")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_error() {
    assert_eq!(classify(input_with_prompt("handle the error gracefully")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_crash() {
    assert_eq!(classify(input_with_prompt("app crash on startup")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_broken() {
    assert_eq!(classify(input_with_prompt("the build is broken")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_issue() {
    assert_eq!(classify(input_with_prompt("address the issue in routing")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_patch() {
    assert_eq!(classify(input_with_prompt("apply the patch upstream")), Some(TaskType::Bugfix));
}

#[test]
fn classifies_bugfix_keyword_regression() {
    assert_eq!(classify(input_with_prompt("this is a regression from v2")), Some(TaskType::Bugfix));
}

// ─── Refactor ─────────────────────────────────────────────────────────────────

#[test]
fn classifies_refactor_keyword_refactor() {
    assert_eq!(classify(input_with_prompt("refactor the database layer")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_cleanup() {
    assert_eq!(classify(input_with_prompt("cleanup dead code")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_reorganize() {
    assert_eq!(classify(input_with_prompt("reorganize the module structure")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_rename() {
    assert_eq!(classify(input_with_prompt("rename the helper functions")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_simplify() {
    assert_eq!(classify(input_with_prompt("simplify the control flow")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_extract() {
    assert_eq!(classify(input_with_prompt("extract a trait from this struct")), Some(TaskType::Refactor));
}

#[test]
fn classifies_refactor_keyword_deduplicate() {
    assert_eq!(classify(input_with_prompt("deduplicate the validation logic")), Some(TaskType::Refactor));
}

// ─── Architecture ─────────────────────────────────────────────────────────────

#[test]
fn classifies_architecture_keyword_architecture() {
    assert_eq!(classify(input_with_prompt("review the architecture")), Some(TaskType::Architecture));
}

#[test]
fn classifies_architecture_keyword_design() {
    assert_eq!(classify(input_with_prompt("design the new API surface")), Some(TaskType::Architecture));
}

#[test]
fn classifies_architecture_keyword_system() {
    assert_eq!(classify(input_with_prompt("the system needs a new queue")), Some(TaskType::Architecture));
}

#[test]
fn classifies_architecture_keyword_infrastructure() {
    assert_eq!(classify(input_with_prompt("set up the infrastructure")), Some(TaskType::Architecture));
}

#[test]
fn classifies_architecture_keyword_migration() {
    assert_eq!(classify(input_with_prompt("plan the database migration")), Some(TaskType::Architecture));
}

#[test]
fn classifies_architecture_keyword_schema() {
    assert_eq!(classify(input_with_prompt("update the schema definition")), Some(TaskType::Architecture));
}

// ─── Release ──────────────────────────────────────────────────────────────────

#[test]
fn classifies_release_keyword_release() {
    assert_eq!(classify(input_with_prompt("prepare the release")), Some(TaskType::Release));
}

#[test]
fn classifies_release_keyword_deploy() {
    assert_eq!(classify(input_with_prompt("deploy to staging")), Some(TaskType::Release));
}

#[test]
fn classifies_release_keyword_publish() {
    assert_eq!(classify(input_with_prompt("publish the crate")), Some(TaskType::Release));
}

#[test]
fn classifies_release_keyword_version() {
    assert_eq!(classify(input_with_prompt("bump the version number")), Some(TaskType::Release));
}

#[test]
fn classifies_release_keyword_tag() {
    assert_eq!(classify(input_with_prompt("create a tag for v3")), Some(TaskType::Release));
}

#[test]
fn classifies_release_keyword_changelog() {
    assert_eq!(classify(input_with_prompt("update the changelog")), Some(TaskType::Release));
}

// ─── Research ─────────────────────────────────────────────────────────────────

#[test]
fn classifies_research_keyword_research() {
    assert_eq!(classify(input_with_prompt("research alternatives")), Some(TaskType::Research));
}

#[test]
fn classifies_research_keyword_investigate() {
    assert_eq!(classify(input_with_prompt("investigate the cause")), Some(TaskType::Research));
}

#[test]
fn classifies_research_keyword_explore() {
    assert_eq!(classify(input_with_prompt("explore new frameworks")), Some(TaskType::Research));
}

#[test]
fn classifies_research_keyword_prototype() {
    assert_eq!(classify(input_with_prompt("build a prototype")), Some(TaskType::Research));
}

#[test]
fn classifies_research_keyword_spike() {
    assert_eq!(classify(input_with_prompt("do a spike on caching")), Some(TaskType::Research));
}

#[test]
fn classifies_research_keyword_experiment() {
    assert_eq!(classify(input_with_prompt("experiment with tokio")), Some(TaskType::Research));
}

// ─── Feature (default fallback) ───────────────────────────────────────────────

#[test]
fn classifies_feature_as_default() {
    assert_eq!(classify(input_with_prompt("add a new button to the UI")), Some(TaskType::Feature));
}

#[test]
fn classifies_feature_for_generic_prompt() {
    assert_eq!(classify(input_with_prompt("implement the dashboard")), Some(TaskType::Feature));
}

// ─── Priority ordering ───────────────────────────────────────────────────────

#[test]
fn security_takes_priority_over_bugfix() {
    // "fix" is a bugfix keyword, but "security" takes priority.
    assert_eq!(
        classify(input_with_prompt("fix the security hole")),
        Some(TaskType::Security)
    );
}

#[test]
fn security_takes_priority_over_refactor() {
    assert_eq!(
        classify(input_with_prompt("refactor the auth module")),
        Some(TaskType::Security)
    );
}

#[test]
fn security_takes_priority_over_release() {
    assert_eq!(
        classify(input_with_prompt("deploy the credential rotation")),
        Some(TaskType::Security)
    );
}

#[test]
fn bugfix_takes_priority_over_refactor() {
    // "fix" is bugfix, "cleanup" is refactor. Bugfix wins.
    assert_eq!(
        classify(input_with_prompt("fix the cleanup routine")),
        Some(TaskType::Bugfix)
    );
}

#[test]
fn bugfix_takes_priority_over_architecture() {
    assert_eq!(
        classify(input_with_prompt("fix the system design flaw")),
        Some(TaskType::Bugfix)
    );
}

#[test]
fn refactor_takes_priority_over_architecture() {
    assert_eq!(
        classify(input_with_prompt("refactor the system layer")),
        Some(TaskType::Refactor)
    );
}

#[test]
fn refactor_takes_priority_over_release() {
    assert_eq!(
        classify(input_with_prompt("cleanup before release")),
        Some(TaskType::Refactor)
    );
}

// ─── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn empty_prompt_defaults_to_feature() {
    assert_eq!(classify(input_with_prompt("")), Some(TaskType::Feature));
}

#[test]
fn no_prompt_defaults_to_feature() {
    let input = HookInput {
        hook_event_name: HookEvent::PreToolUse,
        ..Default::default()
    };
    assert_eq!(classify(input), Some(TaskType::Feature));
}

#[test]
fn case_insensitive_classification() {
    assert_eq!(classify(input_with_prompt("FIX THE BUG")), Some(TaskType::Bugfix));
    assert_eq!(classify(input_with_prompt("SECURITY audit")), Some(TaskType::Security));
    assert_eq!(classify(input_with_prompt("Refactor This Module")), Some(TaskType::Refactor));
}

#[test]
fn classification_uses_tool_name() {
    // "auth" in tool_name should trigger Security even with a benign prompt.
    let input = input_with_prompt_and_tool("do the thing", "auth_checker");
    assert_eq!(classify(input), Some(TaskType::Security));
}

#[test]
fn classification_uses_tool_input() {
    let mut input = input_with_prompt("do the thing");
    input.tool_input = Some(serde_json::json!({"command": "fix the crash"}));
    assert_eq!(classify(input), Some(TaskType::Bugfix));
}

#[test]
fn classification_uses_last_assistant_message() {
    let mut input = input_with_prompt("do the thing");
    input.last_assistant_message = Some("I will refactor the module".to_string());
    assert_eq!(classify(input), Some(TaskType::Refactor));
}

#[test]
fn keyword_as_substring_still_matches() {
    // "authorization" contains "auth", so should classify as Security.
    assert_eq!(
        classify(input_with_prompt("check the authorization logic")),
        Some(TaskType::Security)
    );
}

#[test]
fn release_takes_priority_over_research() {
    // "deploy" is Release, "investigate" is Research.
    // Priority order: Security > Bugfix > Refactor > Architecture > Release > Research.
    // Release is checked before Research, so Release wins.
    assert_eq!(
        classify(input_with_prompt("investigate the deploy pipeline")),
        Some(TaskType::Release)
    );
}

#[test]
fn task_type_is_none_before_classify_runs() {
    let input = input_with_prompt("fix the bug");
    let ctx = LoopContext::new(input);
    assert_eq!(ctx.task_type, None, "task_type should be None before classify stage runs");
}

#[test]
fn combined_text_from_all_sources() {
    // Prompt has no keywords, but tool_name + tool_input + last_assistant_message
    // together form "secret" across fields. However, each field is checked independently
    // as substrings of the joined string. Let's use a keyword in last_assistant_message only.
    let input = HookInput {
        hook_event_name: HookEvent::PreToolUse,
        prompt: Some("do something".into()),
        tool_name: Some("read_file".into()),
        tool_input: Some(serde_json::json!({"path": "/tmp/data"})),
        last_assistant_message: Some("I found a vulnerability".into()),
        ..Default::default()
    };
    assert_eq!(classify(input), Some(TaskType::Security));
}
