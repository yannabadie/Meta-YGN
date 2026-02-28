use axum::extract::State;
use axum::response::Json;
use axum::{Router, routing::post};
use regex::Regex;
use std::sync::LazyLock;

use metaygn_shared::protocol::{HookInput, HookOutput, PermissionDecision};

use crate::app_state::AppState;

// ---------------------------------------------------------------------------
// Security patterns for pre-tool-use
// ---------------------------------------------------------------------------

/// Commands that are always denied (destructive).
static DESTRUCTIVE_PATTERNS: &[&str] = &[
    r"rm\s+-rf\s+/",
    r"sudo\s+rm\s+-rf",
    r"\bmkfs\b",
    r"\bdd\s+if=",
    r"\bshutdown\b",
    r"\breboot\b",
    r":\(\)\s*\{\s*:\|:\s*&\s*\}\s*;?\s*:", // fork bomb
];

/// Commands that require user confirmation (high risk).
static HIGH_RISK_PATTERNS: &[&str] = &[
    r"\bgit\s+push\b",
    r"\bgit\s+reset\s+--hard\b",
    r"\bterraform\s+apply\b",
    r"\bterraform\s+destroy\b",
    r"\bkubectl\s+apply\b",
    r"\bkubectl\s+delete\b",
    r"\bcurl\b.*\|\s*bash",
    r"\bsudo\b",
];

/// Keywords for risk classification of user prompts.
static HIGH_RISK_KEYWORDS: &[&str] = &[
    "auth", "oauth", "token", "secret", "deploy", "payment", "billing",
    "migration", "database", "prod", "production", "security", "delete",
    "terraform", "kubernetes", "docker", "ci/cd", "release",
];

static LOW_RISK_KEYWORDS: &[&str] = &[
    "typo", "rename", "comment", "docs", "readme", "format", "lint", "cleanup",
];

/// Pre-compiled destructive regexes.
static DESTRUCTIVE_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    DESTRUCTIVE_PATTERNS
        .iter()
        .map(|p| Regex::new(p).expect("invalid destructive regex pattern"))
        .collect()
});

/// Pre-compiled high-risk regexes.
static HIGH_RISK_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    HIGH_RISK_PATTERNS
        .iter()
        .map(|p| Regex::new(p).expect("invalid high-risk regex pattern"))
        .collect()
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the command string from tool_input. The command may be in
/// tool_input.command (Bash tool) or tool_input itself might be a string.
fn extract_command(input: &HookInput) -> String {
    if let Some(ref tool_input) = input.tool_input {
        // Try tool_input.command (Bash tool pattern)
        if let Some(cmd) = tool_input.get("command").and_then(|v| v.as_str()) {
            return cmd.to_string();
        }
        // Try tool_input.input (Write/Edit tool pattern)
        if let Some(cmd) = tool_input.get("input").and_then(|v| v.as_str()) {
            return cmd.to_string();
        }
        // Fallback: serialize the entire tool_input
        return tool_input.to_string();
    }
    String::new()
}

/// Classify a command for pre-tool-use gating.
fn classify_command(tool_name: &str, command: &str) -> HookOutput {
    // MCP tools always need confirmation
    if tool_name.starts_with("mcp__") {
        return HookOutput::permission(
            PermissionDecision::Ask,
            format!("MCP tool '{tool_name}' requires user confirmation"),
        );
    }

    // Check destructive patterns
    for re in DESTRUCTIVE_REGEXES.iter() {
        if re.is_match(command) {
            return HookOutput::permission(
                PermissionDecision::Deny,
                format!("Destructive command detected: matched pattern '{}'", re.as_str()),
            );
        }
    }

    // Check high-risk patterns
    for re in HIGH_RISK_REGEXES.iter() {
        if re.is_match(command) {
            return HookOutput::permission(
                PermissionDecision::Ask,
                format!("High-risk command detected: matched pattern '{}'", re.as_str()),
            );
        }
    }

    // Default: allow
    HookOutput::allow()
}

/// Classify a user prompt by risk level.
fn classify_prompt(prompt: &str) -> (&'static str, &'static str) {
    let lower = prompt.to_lowercase();

    for kw in HIGH_RISK_KEYWORDS {
        if lower.contains(kw) {
            return ("high", "Prompt involves sensitive operations; proceed with extra caution.");
        }
    }

    for kw in LOW_RISK_KEYWORDS {
        if lower.contains(kw) {
            return ("low", "Routine change; standard workflow applies.");
        }
    }

    ("medium", "Standard risk; normal review process recommended.")
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

/// POST /hooks/pre-tool-use
async fn pre_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let command = extract_command(&input);

    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "pre_tool_use", &payload).await;

    let output = classify_command(&tool_name, &command);
    Json(output)
}

/// POST /hooks/post-tool-use
async fn post_tool_use(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    // Log the tool output for verification signals
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "post_tool_use", &payload).await;

    // Post-tool-use: emit verification context based on what happened
    let tool_name = input.tool_name.clone().unwrap_or_default();
    let response = input.tool_response.clone().unwrap_or_default();

    let context = if tool_name == "Bash" && response.contains("FAIL") {
        "Test failure detected in Bash output. Review results before proceeding."
    } else if tool_name == "Write" || tool_name == "Edit" {
        "File modification recorded. Verify changes align with intent."
    } else if tool_name.starts_with("mcp__") {
        "MCP tool output recorded. Verify external system state."
    } else {
        "Tool output recorded."
    };

    Json(HookOutput::context(
        "PostToolUse".to_string(),
        context.to_string(),
    ))
}

/// POST /hooks/user-prompt-submit
async fn user_prompt_submit(
    State(state): State<AppState>,
    Json(input): Json<HookInput>,
) -> Json<HookOutput> {
    let prompt = input.prompt.clone().unwrap_or_default();

    // Log event to memory
    let payload = serde_json::to_string(&input).unwrap_or_default();
    let _ = state.memory.log_event("daemon", "user_prompt_submit", &payload).await;

    let (risk, hint) = classify_prompt(&prompt);

    Json(HookOutput::context(
        "UserPromptSubmit".to_string(),
        format!("[risk:{risk}] {hint}"),
    ))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hooks/pre-tool-use", post(pre_tool_use))
        .route("/hooks/post-tool-use", post(post_tool_use))
        .route("/hooks/user-prompt-submit", post(user_prompt_submit))
}
