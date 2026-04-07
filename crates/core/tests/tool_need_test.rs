use metaygn_core::context::LoopContext;
use metaygn_core::stages::tool_need::ToolNeedStage;
use metaygn_core::stages::{Stage, StageResult};
use metaygn_shared::protocol::{HookEvent, HookInput};

/// Helper: build a `HookInput` with tool_name and an optional command in tool_input.
fn input_with_tool(tool_name: Option<&str>, command: Option<&str>) -> HookInput {
    let mut input = HookInput {
        hook_event_name: HookEvent::PreToolUse,
        session_id: Some("test-session".into()),
        cwd: Some("/tmp".into()),
        ..Default::default()
    };
    input.tool_name = tool_name.map(|s| s.to_string());
    if let Some(cmd) = command {
        let mut map = serde_json::Map::new();
        map.insert(
            "command".to_string(),
            serde_json::Value::String(cmd.to_string()),
        );
        input.tool_input = Some(serde_json::Value::Object(map));
    }
    input
}

/// Helper: run the tool_need stage and return (tool_necessary, tool_necessity_reason).
fn assess_tool_need(input: HookInput) -> (bool, Option<String>) {
    let stage = ToolNeedStage;
    let mut ctx = LoopContext::new(input);
    let result = stage.run(&mut ctx);
    assert_eq!(result, StageResult::Continue, "tool_need stage should always return Continue");
    (ctx.tool_necessary, ctx.tool_necessity_reason)
}

// ─── Stage metadata ───────────────────────────────────────────────────────────

#[test]
fn stage_name_is_tool_need() {
    let stage = ToolNeedStage;
    assert_eq!(stage.name(), "tool_need");
}

#[test]
fn stage_always_returns_continue() {
    let stage = ToolNeedStage;
    let mut ctx = LoopContext::new(input_with_tool(None, None));
    assert_eq!(stage.run(&mut ctx), StageResult::Continue);
}

// ─── No tool: tool_necessary = false ─────────────────────────────────────────

#[test]
fn no_tool_name_not_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(None, None));
    assert!(!necessary);
    assert!(reason.is_none());
}

#[test]
fn no_tool_name_with_command_still_not_necessary() {
    // Even if tool_input has a command, no tool_name means not necessary
    let mut input = input_with_tool(None, Some("ls -la"));
    input.tool_name = None;
    let (necessary, reason) = assess_tool_need(input);
    assert!(!necessary);
    assert!(reason.is_none());
}

// ─── Tool present: tool_necessary = true (general case) ─────────────────────

#[test]
fn bash_with_regular_command_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Bash"), Some("cargo test --workspace")));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn read_tool_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Read"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn write_tool_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Write"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn grep_tool_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Grep"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn glob_tool_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Glob"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn edit_tool_is_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Edit"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

#[test]
fn bash_with_complex_command_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("git push origin main")));
    assert!(necessary);
}

#[test]
fn bash_with_no_command_is_necessary() {
    // tool_name = Bash but no command in tool_input => still necessary (not echo-only)
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Bash"), None));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

// ─── Echo-only detection: tool_necessary = false ─────────────────────────────

#[test]
fn echo_simple_is_not_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo hello world")));
    assert!(!necessary);
    assert!(reason.as_deref().unwrap().contains("echo"));
}

#[test]
fn echo_single_arg_is_not_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo test")));
    assert!(!necessary);
}

#[test]
fn printf_simple_is_not_necessary() {
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Bash"), Some("printf hello")));
    assert!(!necessary);
    assert!(reason.as_deref().unwrap().contains("echo"));
}

#[test]
fn printf_format_string_is_not_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("printf '%s\\n' hello")));
    assert!(!necessary);
}

// ─── Echo with side-effects: tool_necessary = true ──────────────────────────

#[test]
fn echo_with_redirect_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo secret > leak.txt")));
    assert!(necessary);
}

#[test]
fn echo_with_append_redirect_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo data >> file.txt")));
    assert!(necessary);
}

#[test]
fn echo_with_pipe_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo hello | wc -c")));
    assert!(necessary);
}

#[test]
fn echo_with_ampersand_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo hello & sleep 1")));
    assert!(necessary);
}

#[test]
fn echo_with_semicolon_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echo hello; rm -rf /")));
    assert!(necessary);
}

#[test]
fn printf_with_redirect_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("printf hello > file.txt")));
    assert!(necessary);
}

#[test]
fn printf_with_pipe_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("printf hello | cat")));
    assert!(necessary);
}

// ─── Echo-like but not echo ──────────────────────────────────────────────────

#[test]
fn echo_without_space_is_treated_as_regular_command() {
    // "echoworld" does not start with "echo ", so it's a regular tool invocation
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("echoworld")));
    assert!(necessary);
}

#[test]
fn echo_with_leading_whitespace() {
    // The is_echo_only function trims, so "  echo hello" should be detected
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("  echo hello")));
    assert!(!necessary);
}

// ─── Default context before tool_need runs ───────────────────────────────────

#[test]
fn default_tool_necessary_is_false() {
    let ctx = LoopContext::new(input_with_tool(None, None));
    assert!(!ctx.tool_necessary, "default tool_necessary should be false");
}

#[test]
fn default_tool_necessity_reason_is_none() {
    let ctx = LoopContext::new(input_with_tool(None, None));
    assert!(ctx.tool_necessity_reason.is_none(), "default reason should be None");
}

// ─── Non-Bash tool with command in tool_input ────────────────────────────────

#[test]
fn non_bash_tool_with_echo_command_is_still_necessary() {
    // Only Bash tool triggers echo-only detection
    let (necessary, reason) = assess_tool_need(input_with_tool(Some("Read"), Some("echo hello")));
    assert!(necessary);
    assert_eq!(reason.as_deref(), Some("tool invocation detected"));
}

// ─── Empty and edge-case commands ────────────────────────────────────────────

#[test]
fn bash_with_empty_command_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("")));
    assert!(necessary);
}

#[test]
fn bash_with_whitespace_only_command_is_necessary() {
    let (necessary, _) = assess_tool_need(input_with_tool(Some("Bash"), Some("   ")));
    assert!(necessary);
}
