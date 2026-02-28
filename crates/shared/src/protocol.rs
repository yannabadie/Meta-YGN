use serde::{Deserialize, Serialize};

/// Events that can trigger hooks in the Claude Code lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HookEvent {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    Stop,
    PreCompact,
    SessionEnd,
}

/// Input payload sent to a hook when it is invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    pub hook_event_name: HookEvent,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<String>,
    pub prompt: Option<String>,
    pub error: Option<String>,
    pub last_assistant_message: Option<String>,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub trigger: Option<String>,
}

/// A hook's permission decision for tool-use gating.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

/// Hook-specific output fields returned by a hook script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_event_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<PermissionDecision>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

/// Top-level output structure returned by a hook.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

impl HookOutput {
    /// Create an allow output (empty/minimal).
    pub fn allow() -> Self {
        Self::default()
    }

    /// Create a permission decision output with a reason.
    pub fn permission(decision: PermissionDecision, reason: String) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput {
                permission_decision: Some(decision),
                permission_decision_reason: Some(reason),
                ..Default::default()
            }),
        }
    }

    /// Create an output with additional context for a given event.
    pub fn context(event: String, message: String) -> Self {
        Self {
            hook_specific_output: Some(HookSpecificOutput {
                hook_event_name: Some(event),
                additional_context: Some(message),
                ..Default::default()
            }),
        }
    }
}
