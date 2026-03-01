use metaygn_shared::protocol::{HookEvent, HookInput, HookOutput, PermissionDecision};

#[test]
fn hook_input_deserializes_pre_tool_use() {
    let json = r#"{
        "hook_event_name": "PreToolUse",
        "session_id": "sess-001",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    }"#;

    let input: HookInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.hook_event_name, HookEvent::PreToolUse);
    assert_eq!(input.tool_name.as_deref(), Some("Bash"));
    assert_eq!(input.session_id.as_deref(), Some("sess-001"));
    assert!(input.tool_input.is_some());
}

#[test]
fn hook_output_serializes_deny() {
    let output = HookOutput::permission(PermissionDecision::Deny, "dangerous command".to_string());

    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("deny"), "expected 'deny' in: {json}");
    assert!(
        json.contains("dangerous command"),
        "expected reason in: {json}"
    );
}

#[test]
fn hook_output_serializes_allow_by_empty() {
    let output = HookOutput::allow();
    let json = serde_json::to_string(&output).unwrap();
    // allow() should produce minimal JSON (empty object or null hookSpecificOutput)
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    // hookSpecificOutput should be absent or null
    assert!(
        value.get("hookSpecificOutput").is_none() || value["hookSpecificOutput"].is_null(),
        "expected empty/null hookSpecificOutput in: {json}"
    );
}
