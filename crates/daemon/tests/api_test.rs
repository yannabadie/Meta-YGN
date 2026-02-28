use std::net::SocketAddr;

use serde_json::{json, Value};

async fn start_test_server() -> SocketAddr {
    let app = metaygn_daemon::build_app().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // small delay for server startup
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    addr
}

#[tokio::test]
async fn health_returns_ok() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/health");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], "0.1.0");
    assert_eq!(body["kernel"], "verified");
}

#[tokio::test]
async fn hook_pre_tool_use_allows_safe() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/pre-tool-use");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "ls -la"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    // For safe commands, either no hookSpecificOutput or no permissionDecision
    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert!(
        decision.is_none() || decision == Some("allow"),
        "Expected allow or absent decision for safe command, got: {body:?}"
    );
}

#[tokio::test]
async fn hook_pre_tool_use_denies_destructive() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/pre-tool-use");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "rm -rf /"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert_eq!(decision, Some("deny"), "Expected deny for destructive command, got: {body:?}");
}

#[tokio::test]
async fn hook_pre_tool_use_asks_high_risk() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/pre-tool-use");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "git push"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert_eq!(decision, Some("ask"), "Expected ask for high-risk command, got: {body:?}");
}

#[tokio::test]
async fn hook_user_prompt_submit_classifies() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/user-prompt-submit");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "UserPromptSubmit",
            "prompt": "deploy to production"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let context = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        context.to_lowercase().contains("high"),
        "Expected high risk classification for 'deploy to production', got context: {context}"
    );
}

// ---------------------------------------------------------------------------
// New tests for Task 15: control loop + guard pipeline wiring
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pre_tool_use_uses_guard_pipeline() {
    // A destructive command should be denied by the guard pipeline.
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/pre-tool-use");
    let client = reqwest::Client::new();

    // "sudo rm -rf" triggers DestructiveGuard (score 0) -> deny
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "sudo rm -rf /important"}
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert_eq!(
        decision,
        Some("deny"),
        "Expected deny from guard pipeline for destructive command, got: {body:?}"
    );

    // Verify the reason mentions the guard
    let reason = body
        .pointer("/hookSpecificOutput/permissionDecisionReason")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        reason.contains("guard") || reason.contains("destructive") || reason.contains("Destructive"),
        "Expected guard-related reason, got: {reason}"
    );
}

#[tokio::test]
async fn user_prompt_returns_strategy() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/user-prompt-submit");
    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "UserPromptSubmit",
            "prompt": "fix the login bug in the authentication module"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    let context = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Should contain strategy recommendation from the control loop
    assert!(
        context.contains("strategy:"),
        "Expected strategy recommendation in context, got: {context}"
    );
    // Should contain risk level
    assert!(
        context.contains("risk:"),
        "Expected risk level in context, got: {context}"
    );
    // Should contain budget info
    assert!(
        context.contains("budget:"),
        "Expected budget info in context, got: {context}"
    );
    // Should contain task type
    assert!(
        context.contains("task:"),
        "Expected task type in context, got: {context}"
    );
}

#[tokio::test]
async fn analyze_returns_full_context() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/analyze");
    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "cargo test"},
            "prompt": "run all the tests"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    // Full LoopContext should be present
    assert!(body.get("risk").is_some(), "Expected 'risk' field in analyze response: {body:?}");
    assert!(body.get("difficulty").is_some(), "Expected 'difficulty' field: {body:?}");
    assert!(body.get("competence").is_some(), "Expected 'competence' field: {body:?}");
    assert!(body.get("strategy").is_some(), "Expected 'strategy' field: {body:?}");
    assert!(body.get("decision").is_some(), "Expected 'decision' field: {body:?}");
    assert!(body.get("budget").is_some(), "Expected 'budget' field: {body:?}");
    assert!(body.get("metacog_vector").is_some(), "Expected 'metacog_vector' field: {body:?}");
    assert!(body.get("lessons").is_some(), "Expected 'lessons' field: {body:?}");
    assert!(body.get("input").is_some(), "Expected 'input' field: {body:?}");

    // Verify difficulty is a reasonable float
    let difficulty = body["difficulty"].as_f64().unwrap();
    assert!(difficulty >= 0.0 && difficulty <= 1.0, "difficulty out of range: {difficulty}");

    // Verify competence is a reasonable float
    let competence = body["competence"].as_f64().unwrap();
    assert!(competence >= 0.0 && competence <= 1.0, "competence out of range: {competence}");
}

#[tokio::test]
async fn stop_returns_enforcement_hint() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/stop");
    let client = reqwest::Client::new();

    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "Stop",
            "prompt": "session ending"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    let context = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Should contain decision info
    assert!(
        context.contains("decision:"),
        "Expected decision in stop response, got: {context}"
    );
    // Should contain metacog vector
    assert!(
        context.contains("metacog:"),
        "Expected metacognitive vector in stop response, got: {context}"
    );
    // Should mention proof packet
    assert!(
        context.to_lowercase().contains("proof packet"),
        "Expected proof packet mention in stop response, got: {context}"
    );
}
