use std::net::SocketAddr;

use metaygn_daemon::app_state::AppState;
use serde_json::{Value, json};

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
    assert_eq!(
        decision,
        Some("deny"),
        "Expected deny for destructive command, got: {body:?}"
    );
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
    assert_eq!(
        decision,
        Some("ask"),
        "Expected ask for high-risk command, got: {body:?}"
    );
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
        reason.contains("guard")
            || reason.contains("destructive")
            || reason.contains("Destructive"),
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
        context.contains("Strategy:"),
        "Expected strategy recommendation in context, got: {context}"
    );
    // Should contain risk level
    assert!(
        context.contains("Risk:"),
        "Expected risk level in context, got: {context}"
    );
    // Should contain budget info
    assert!(
        context.contains("Budget:"),
        "Expected budget info in context, got: {context}"
    );
    // Should contain task type
    assert!(
        context.contains("Task:"),
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
    assert!(
        body.get("risk").is_some(),
        "Expected 'risk' field in analyze response: {body:?}"
    );
    assert!(
        body.get("difficulty").is_some(),
        "Expected 'difficulty' field: {body:?}"
    );
    assert!(
        body.get("competence").is_some(),
        "Expected 'competence' field: {body:?}"
    );
    assert!(
        body.get("strategy").is_some(),
        "Expected 'strategy' field: {body:?}"
    );
    assert!(
        body.get("decision").is_some(),
        "Expected 'decision' field: {body:?}"
    );
    assert!(
        body.get("budget").is_some(),
        "Expected 'budget' field: {body:?}"
    );
    assert!(
        body.get("metacog_vector").is_some(),
        "Expected 'metacog_vector' field: {body:?}"
    );
    assert!(
        body.get("lessons").is_some(),
        "Expected 'lessons' field: {body:?}"
    );
    assert!(
        body.get("input").is_some(),
        "Expected 'input' field: {body:?}"
    );

    // Verify difficulty is a reasonable float
    let difficulty = body["difficulty"].as_f64().unwrap();
    assert!(
        difficulty >= 0.0 && difficulty <= 1.0,
        "difficulty out of range: {difficulty}"
    );

    // Verify competence is a reasonable float
    let competence = body["competence"].as_f64().unwrap();
    assert!(
        competence >= 0.0 && competence <= 1.0,
        "competence out of range: {competence}"
    );
}

// ---------------------------------------------------------------------------
// Phase 3: sandbox and profiler endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sandbox_exec_python() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/sandbox/exec");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "language": "python",
            "code": "print('hello')",
            "timeout_ms": 5000
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    // If python is not available, stderr will contain a spawn error.
    // Either way, we should get a well-formed SandboxResult.
    assert!(
        body.get("success").is_some(),
        "Expected 'success' field in sandbox result: {body:?}"
    );
    assert!(
        body.get("duration_ms").is_some(),
        "Expected 'duration_ms' field in sandbox result: {body:?}"
    );
    // If python is available, stdout should contain "hello"
    if body["success"].as_bool() == Some(true) {
        let stdout = body["stdout"].as_str().unwrap_or("");
        assert!(
            stdout.contains("hello"),
            "Expected 'hello' in stdout, got: {stdout}"
        );
    }
}

#[tokio::test]
async fn profiler_fatigue_starts_zero() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/profiler/fatigue");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let score = body["score"].as_f64().unwrap();
    assert!(
        score.abs() < f64::EPSILON,
        "Expected score ~0 for fresh profiler, got {score}"
    );
    assert_eq!(body["high_friction"], false);
}

#[tokio::test]
async fn profiler_signal_increases_score() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();

    // Send a short prompt signal -- "fix" is 3 chars, well below threshold
    let url = format!("http://{addr}/profiler/signal");
    let resp = client
        .post(&url)
        .json(&json!({
            "signal_type": "prompt",
            "prompt": "fix"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let score = body["score"].as_f64().unwrap();
    assert!(
        score > 0.0,
        "Expected score > 0 after short prompt signal, got {score}"
    );
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

// ---------------------------------------------------------------------------
// Phase 4: graph memory, heuristics, forge endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn graph_insert_and_search() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();

    // Insert a node
    let url = format!("http://{addr}/memory/nodes");
    let resp = client
        .post(&url)
        .json(&json!({
            "id": "node-1",
            "node_type": "Task",
            "scope": "Session",
            "label": "Fix authentication bug",
            "content": "The login form rejects valid credentials due to a hashing mismatch",
            "embedding": null,
            "created_at": "2025-01-01T00:00:00Z",
            "access_count": 0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["id"], "node-1");

    // Search for the node via FTS
    let url = format!("http://{addr}/memory/graph/search");
    let resp = client
        .post(&url)
        .json(&json!({ "query": "authentication", "limit": 5 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let nodes = body["nodes"].as_array().expect("expected nodes array");
    assert!(
        !nodes.is_empty(),
        "Expected at least one node matching 'authentication', got empty"
    );
    assert_eq!(nodes[0]["id"], "node-1");

    // Verify stats
    let url = format!("http://{addr}/memory/graph/stats");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["node_count"], 1);
    assert_eq!(body["edge_count"], 0);
}

#[tokio::test]
async fn heuristics_evolve() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();

    // Record an outcome first so evolution has data
    let url = format!("http://{addr}/heuristics/outcome");
    let resp = client
        .post(&url)
        .json(&json!({
            "session_id": "sess-1",
            "task_type": "bugfix",
            "risk_level": "medium",
            "strategy_used": "step_by_step",
            "success": true,
            "tokens_consumed": 5000,
            "duration_ms": 30000,
            "errors_encountered": 0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Trigger evolution
    let url = format!("http://{addr}/heuristics/evolve");
    let resp = client.post(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert!(
        body.get("best").is_some(),
        "Expected 'best' in evolve response: {body:?}"
    );

    // Verify population grew
    let url = format!("http://{addr}/heuristics/population");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let size = body["size"].as_u64().unwrap();
    assert!(
        size >= 2,
        "Expected population size >= 2 after evolve, got {size}"
    );
    assert!(body["generation"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn forge_list_templates() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/forge/templates");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let templates = body["templates"]
        .as_array()
        .expect("expected templates array");
    assert_eq!(
        templates.len(),
        5,
        "Expected 5 templates, got {}: {body:?}",
        templates.len()
    );

    // Verify the 5 expected template names are present
    let names: Vec<&str> = templates
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"grep-pattern-checker"));
    assert!(names.contains(&"import-validator"));
    assert!(names.contains(&"json-validator"));
    assert!(names.contains(&"file-exists-checker"));
    assert!(names.contains(&"syntax-checker"));
}

// ---------------------------------------------------------------------------
// Completion verifier integration test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn stop_hook_catches_false_completion() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/stop");
    let client = reqwest::Client::new();

    // Send a Stop event where Claude claims "Done!" but mentions a file that doesn't exist
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "Stop",
            "last_assistant_message": "Done! I created `fake/nonexistent/module.rs` with all the changes.",
            "cwd": "."
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

    // Should detect the false completion and block it
    assert!(
        context.contains("COMPLETION CHECK FAILED"),
        "Expected completion check failure for missing file, got: {context}"
    );
    assert!(
        context.contains("NOT FOUND"),
        "Expected NOT FOUND in failure message, got: {context}"
    );
}

// ---------------------------------------------------------------------------
// Token Budget Tracker integration tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn budget_starts_at_zero() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/budget");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["consumed_tokens"], 0);
    assert_eq!(body["consumed_cost_usd"], 0.0);
}

#[tokio::test]
async fn budget_consume_updates() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();

    // Consume some tokens
    let url = format!("http://{addr}/budget/consume");
    let resp = client
        .post(&url)
        .json(&json!({ "tokens": 500, "cost_usd": 0.005 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["consumed_tokens"], 500);

    // Verify via GET
    let url = format!("http://{addr}/budget");
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["consumed_tokens"], 500);
    let cost = body["consumed_cost_usd"].as_f64().unwrap();
    assert!((cost - 0.005).abs() < 1e-9);
}

#[tokio::test]
async fn hook_responses_include_budget() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();

    // Submit a prompt — the response should contain "[budget:"
    let url = format!("http://{addr}/hooks/user-prompt-submit");
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "UserPromptSubmit",
            "prompt": "implement the login feature with full test coverage"
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
        context.contains("[budget:"),
        "Expected '[budget:' in hook response additionalContext, got: {context}"
    );
}

// ---------------------------------------------------------------------------
// Test Integrity Guard integration test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn pre_tool_use_catches_test_weakening() {
    let addr = start_test_server().await;
    let url = format!("http://{addr}/hooks/pre-tool-use");
    let client = reqwest::Client::new();

    // Edit a test file, removing an assertion — should trigger Ask
    let resp = client
        .post(&url)
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Edit",
            "tool_input": {
                "file_path": "tests/my_test.rs",
                "old_string": "#[test]\nfn it_works() {\n    assert!(result.is_ok());\n    assert_eq!(result.unwrap(), 42);\n}",
                "new_string": "#[test]\nfn it_works() {\n    // looks good\n}"
            }
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
        Some("ask"),
        "Expected ask for test weakening edit, got: {body:?}"
    );

    let reason = body
        .pointer("/hookSpecificOutput/permissionDecisionReason")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        reason.contains("TEST INTEGRITY WARNING"),
        "Expected test integrity warning in reason, got: {reason}"
    );

    let context = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        context.contains("assertion(s) removed"),
        "Expected assertion removal detail in context, got: {context}"
    );
}

// ---------------------------------------------------------------------------
// Task 4: Latency tracking in hook responses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hook_responses_include_latency() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/hooks/user-prompt-submit"))
        .json(&json!({
            "hook_event_name": "UserPromptSubmit",
            "prompt": "fix typo"
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let ctx = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(ctx.contains("latency:"), "Should include latency: {}", ctx);
}

// ---------------------------------------------------------------------------
// Task 3: Pre-tool-use risk classification fix
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Task 3 (v0.6.0): /admin/shutdown endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn admin_shutdown_responds_ok() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/admin/shutdown"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert!(
        body["message"].as_str().unwrap().contains("Shutdown"),
        "Expected shutdown message, got: {body:?}"
    );
}

#[tokio::test]
async fn pre_tool_use_safe_command_not_high_risk() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/hooks/pre-tool-use"))
        .json(&json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "ls -la"}
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let ctx = body
        .pointer("/hookSpecificOutput/additionalContext")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // "ls -la" should be low risk, not high
    assert!(
        !ctx.contains("risk:high"),
        "ls -la should not be HIGH risk: {}",
        ctx
    );
}

// ---------------------------------------------------------------------------
// Task 5: Context pruning service endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_passes_clean_messages() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/proxy/anthropic"))
        .json(&json!({
            "messages": [
                {"role": "user", "content": "hello"},
                {"role": "assistant", "content": "hi there"}
            ],
            "model": "claude-sonnet",
            "max_tokens": 1000
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["pruned"], false);
    assert_eq!(body["recovery_injected"], false);
    assert_eq!(body["tokens_removed"], 0);
    assert!(body["reason"].is_null());
    assert_eq!(body["messages"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn proxy_prunes_error_loop() {
    let addr = start_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://{addr}/proxy/anthropic"))
        .json(&json!({
            "messages": [
                {"role": "user", "content": "fix this"},
                {"role": "assistant", "content": "Error: compilation failed"},
                {"role": "user", "content": "try again"},
                {"role": "assistant", "content": "Error: same compilation failed"},
                {"role": "user", "content": "please fix"},
                {"role": "assistant", "content": "Error: still failing"}
            ],
            "model": "claude-sonnet",
            "max_tokens": 1000
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["pruned"], true);
    assert!(body["recovery_injected"].as_bool().unwrap());
    assert!(body["reason"].as_str().unwrap().contains("ALETHEIA"));
    // Pruned messages should be shorter than the original 6
    assert!(body["messages"].as_array().unwrap().len() < 6);
}

// ---------------------------------------------------------------------------
// Task 8 (v0.7.0): Cross-session learning persistence roundtrip
// ---------------------------------------------------------------------------

#[tokio::test]
async fn heuristic_persistence_roundtrip() {
    // 1. Create AppState (in-memory)
    let state = AppState::new_in_memory().await.unwrap();

    // 2. Record an outcome and persist it via the memory store
    let outcome_id = uuid::Uuid::new_v4().to_string();
    state
        .memory
        .save_outcome(
            &outcome_id,
            "sess-rt",
            "bugfix",
            "medium",
            "step_by_step",
            true,
            3000,
            15000,
            0,
        )
        .await
        .unwrap();

    {
        let mut evolver = state.evolver.lock().unwrap();
        evolver.record_outcome(metaygn_core::heuristics::fitness::SessionOutcome {
            session_id: "sess-rt".into(),
            task_type: "bugfix".into(),
            risk_level: "medium".into(),
            strategy_used: "step_by_step".into(),
            success: true,
            tokens_consumed: 3000,
            duration_ms: 15000,
            errors_encountered: 0,
        });

        // 3. Evolve via evolver directly
        evolver.evolve_generation();

        // 4. Check population > 1 (seed + mutated child)
        assert!(
            evolver.population_size() > 1,
            "Expected population > 1 after evolve, got {}",
            evolver.population_size()
        );

        // Persist the best version
        if let Some(best) = evolver.best() {
            state
                .memory
                .save_heuristic(
                    &best.id,
                    best.generation,
                    best.parent_id.as_deref(),
                    &serde_json::to_string(&best.fitness).unwrap(),
                    &serde_json::to_string(&best.risk_weights).unwrap(),
                    &serde_json::to_string(&best.strategy_scores).unwrap(),
                    &best.created_at,
                )
                .await
                .unwrap();
        }
    }

    // 5. Verify persistence: load back from SQLite
    let versions = state.memory.load_heuristics().await.unwrap();
    assert!(
        !versions.is_empty(),
        "Expected at least one heuristic version persisted"
    );

    let outcomes = state.memory.load_recent_outcomes(10).await.unwrap();
    assert!(
        !outcomes.is_empty(),
        "Expected at least one outcome persisted"
    );
}
