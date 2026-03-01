//! Integration E2E test harness for MetaYGN v0.10.0
//!
//! Starts an in-memory daemon and POSTs hooks in sequence to verify the full
//! circuit works end-to-end: classification -> guard -> post-processing ->
//! graph memory -> replay -> heuristics.

use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Start an in-memory daemon and return (client, base_url).
async fn start_test_daemon() -> (Client, String) {
    let app = metaygn_daemon::build_app().await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // Small delay for server startup
    tokio::time::sleep(Duration::from_millis(50)).await;
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    (client, format!("http://{addr}"))
}

/// POST a hook and return the response body as JSON.
async fn post_hook(client: &Client, base: &str, path: &str, body: Value) -> Value {
    let resp = client
        .post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Expected 200 for {path}, got {}",
        resp.status()
    );
    resp.json().await.unwrap()
}

/// GET and return the response body as JSON.
async fn get_json(client: &Client, base: &str, path: &str) -> Value {
    let resp = client
        .get(format!("{base}{path}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Expected 200 for GET {path}, got {}",
        resp.status()
    );
    resp.json().await.unwrap()
}

// ---------------------------------------------------------------------------
// Test 1: Full session lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_session_lifecycle() {
    let (client, base) = start_test_daemon().await;

    // 1. user_prompt_submit -- classify the task
    let body = post_hook(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "e2e-session-1",
            "prompt": "fix the authentication bug in login.rs"
        }),
    )
    .await;
    assert!(
        body.get("hookSpecificOutput").is_some(),
        "user_prompt_submit should return hookSpecificOutput, got: {body:?}"
    );
    let ctx = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        ctx.contains("Risk:") && ctx.contains("Strategy:"),
        "user_prompt_submit should return risk and strategy, got: {ctx}"
    );

    // 2. pre_tool_use (safe command)
    let body = post_hook(
        &client,
        &base,
        "/hooks/pre-tool-use",
        json!({
            "hook_event_name": "PreToolUse",
            "session_id": "e2e-session-1",
            "tool_name": "Bash",
            "tool_input": {"command": "cargo test"}
        }),
    )
    .await;
    // Safe command should NOT be denied
    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert!(
        decision.is_none() || decision != Some("deny"),
        "cargo test should not be denied, got: {body:?}"
    );

    // 3. post_tool_use (success)
    let body = post_hook(
        &client,
        &base,
        "/hooks/post-tool-use",
        json!({
            "hook_event_name": "PostToolUse",
            "session_id": "e2e-session-1",
            "tool_name": "Bash",
            "tool_response": "test result: ok. 42 passed; 0 failed"
        }),
    )
    .await;
    assert!(
        body.get("hookSpecificOutput").is_some(),
        "post_tool_use should return hookSpecificOutput, got: {body:?}"
    );

    // 4. stop
    let body = post_hook(
        &client,
        &base,
        "/hooks/stop",
        json!({
            "hook_event_name": "Stop",
            "session_id": "e2e-session-1",
            "last_assistant_message": "I've fixed the authentication bug. All 42 tests pass."
        }),
    )
    .await;
    assert!(
        body.get("hookSpecificOutput").is_some(),
        "stop should return hookSpecificOutput, got: {body:?}"
    );

    // Wait for async post-processing (graph node insertions, outcome recording)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 5. Verify graph was populated (Task node from user_prompt_submit,
    //    Evidence node from post_tool_use, Decision node from stop)
    let stats = get_json(&client, &base, "/memory/graph/stats").await;
    let node_count = stats
        .get("node_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(
        node_count > 0,
        "graph should have nodes after full session, got {node_count}"
    );

    // 6. Verify replay was recorded
    let sessions = get_json(&client, &base, "/replay/sessions").await;
    let session_list = sessions
        .get("sessions")
        .and_then(|v| v.as_array())
        .expect("replay sessions should be an array");
    assert!(
        !session_list.is_empty(),
        "replay should have recorded the session"
    );
    // Verify the correct session ID is in the replay
    let has_our_session = session_list
        .iter()
        .any(|s| s.get("session_id").and_then(|v| v.as_str()) == Some("e2e-session-1"));
    assert!(
        has_our_session,
        "replay should contain session e2e-session-1, got: {session_list:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 2: Guard blocks destructive command
// ---------------------------------------------------------------------------

#[tokio::test]
async fn guard_blocks_destructive_command() {
    let (client, base) = start_test_daemon().await;

    let body = post_hook(
        &client,
        &base,
        "/hooks/pre-tool-use",
        json!({
            "hook_event_name": "PreToolUse",
            "session_id": "guard-test",
            "tool_name": "Bash",
            "tool_input": {"command": "rm -rf /"}
        }),
    )
    .await;

    let decision = body
        .pointer("/hookSpecificOutput/permissionDecision")
        .and_then(|v| v.as_str());
    assert_eq!(
        decision,
        Some("deny"),
        "destructive command 'rm -rf /' should be denied, got: {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Session context persists across hooks
// ---------------------------------------------------------------------------

#[tokio::test]
async fn session_context_persists_across_hooks() {
    let (client, base) = start_test_daemon().await;

    // Classify as security task
    post_hook(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "persist-test",
            "prompt": "fix the SQL injection vulnerability in the authentication module"
        }),
    )
    .await;

    // Tool use in same session
    post_hook(
        &client,
        &base,
        "/hooks/pre-tool-use",
        json!({
            "hook_event_name": "PreToolUse",
            "session_id": "persist-test",
            "tool_name": "Edit",
            "tool_input": {"file_path": "src/auth.rs", "old_string": "bad", "new_string": "good"}
        }),
    )
    .await;

    // Stop should have context from the classification (risk, strategy, etc.)
    let body = post_hook(
        &client,
        &base,
        "/hooks/stop",
        json!({
            "hook_event_name": "Stop",
            "session_id": "persist-test",
            "last_assistant_message": "Fixed the SQL injection."
        }),
    )
    .await;

    let context = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // The stop handler feeds session state (risk, strategy, task_type) back
    // into the LoopContext, so the output should contain decision/metacog/lessons
    // from the enriched session data.
    assert!(
        context.contains("decision:")
            || context.contains("metacog:")
            || context.contains("lessons:"),
        "stop response should contain session context data, got: {context}"
    );
}

// ---------------------------------------------------------------------------
// Test 4: Post-tool-use records error evidence in graph
// ---------------------------------------------------------------------------

#[tokio::test]
async fn post_tool_use_records_error_evidence() {
    let (client, base) = start_test_daemon().await;

    // POST post_tool_use with an error response
    post_hook(
        &client,
        &base,
        "/hooks/post-tool-use",
        json!({
            "hook_event_name": "PostToolUse",
            "session_id": "error-evidence-test",
            "tool_name": "Bash",
            "tool_response": "Error: command not found"
        }),
    )
    .await;

    // Wait for async post-processing to insert Evidence node
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Graph should have an Evidence node
    let stats = get_json(&client, &base, "/memory/graph/stats").await;
    let node_count = stats
        .get("node_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(
        node_count > 0,
        "graph should have an Evidence node after error post_tool_use, got {node_count}"
    );

    // Verify we can find the evidence via search
    let search_result = post_hook(
        &client,
        &base,
        "/memory/graph/search",
        json!({ "query": "error", "limit": 5 }),
    )
    .await;
    let nodes = search_result
        .get("nodes")
        .and_then(|v| v.as_array())
        .expect("search should return nodes array");
    assert!(
        !nodes.is_empty(),
        "search for 'error' should find the Evidence node"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Stop records heuristic outcome
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stop_records_heuristic_outcome() {
    let (client, base) = start_test_daemon().await;

    // Run a full session: user_prompt_submit + post_tool_use + stop
    post_hook(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "heuristic-test",
            "prompt": "implement a simple utility function"
        }),
    )
    .await;

    post_hook(
        &client,
        &base,
        "/hooks/post-tool-use",
        json!({
            "hook_event_name": "PostToolUse",
            "session_id": "heuristic-test",
            "tool_name": "Bash",
            "tool_response": "test result: ok. 5 passed; 0 failed"
        }),
    )
    .await;

    post_hook(
        &client,
        &base,
        "/hooks/stop",
        json!({
            "hook_event_name": "Stop",
            "session_id": "heuristic-test",
            "last_assistant_message": "Done. All tests pass."
        }),
    )
    .await;

    // Wait for async post-processing (after_stop records outcome)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // GET /heuristics/population should show the population exists
    let pop = get_json(&client, &base, "/heuristics/population").await;
    let size = pop.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(
        size >= 1,
        "heuristic population should have at least 1 member (seed), got {size}"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Health endpoint sanity check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_endpoint_sanity_check() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/health").await;
    assert_eq!(body["status"], "ok", "health status should be 'ok'");
    assert_eq!(
        body["kernel"], "verified",
        "kernel should be 'verified'"
    );
    assert!(
        body.get("version").is_some(),
        "health should report a version"
    );
}

// ---------------------------------------------------------------------------
// Test 7: Multiple hooks in same session update fatigue profiler
// ---------------------------------------------------------------------------

#[tokio::test]
async fn session_errors_increase_fatigue() {
    let (client, base) = start_test_daemon().await;

    // Check initial fatigue score
    let initial = get_json(&client, &base, "/profiler/fatigue").await;
    let initial_score = initial["score"].as_f64().unwrap_or(0.0);

    // Submit a prompt (this records a fatigue signal)
    post_hook(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "fatigue-test",
            "prompt": "fix"
        }),
    )
    .await;

    // Send a post_tool_use with an error (this records fatigue error signal)
    post_hook(
        &client,
        &base,
        "/hooks/post-tool-use",
        json!({
            "hook_event_name": "PostToolUse",
            "session_id": "fatigue-test",
            "tool_name": "Bash",
            "tool_response": "Error: compilation failed"
        }),
    )
    .await;

    // Fatigue score should have increased
    let after = get_json(&client, &base, "/profiler/fatigue").await;
    let after_score = after["score"].as_f64().unwrap_or(0.0);
    assert!(
        after_score > initial_score,
        "fatigue score should increase after error, was {initial_score}, now {after_score}"
    );
}
