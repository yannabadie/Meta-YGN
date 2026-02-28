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
