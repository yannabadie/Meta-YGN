//! Endpoint coverage tests for MetaYGN daemon API.
//!
//! Tests GET endpoints that lack dedicated coverage, verifying each returns
//! HTTP 200 with valid JSON containing the expected response fields.

use reqwest::Client;
use serde_json::{Value, json};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers (same pattern as integration_e2e.rs)
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

/// GET and return the response body as JSON, asserting HTTP 200.
async fn get_json(client: &Client, base: &str, path: &str) -> Value {
    let resp = client.get(format!("{base}{path}")).send().await.unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Expected 200 for GET {path}, got {}",
        resp.status()
    );
    resp.json().await.unwrap()
}

/// POST a JSON body and return the response body as JSON, asserting HTTP 200.
async fn post_json(client: &Client, base: &str, path: &str, body: Value) -> Value {
    let resp = client
        .post(format!("{base}{path}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Expected 200 for POST {path}, got {}",
        resp.status()
    );
    resp.json().await.unwrap()
}

// ---------------------------------------------------------------------------
// Test: GET /budget/{session_id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_budget_session() {
    let (client, base) = start_test_daemon().await;

    // First create the session by posting a hook so it exists
    post_json(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "budget-test-session",
            "prompt": "hello world"
        }),
    )
    .await;

    let body = get_json(&client, &base, "/budget/budget-test-session").await;

    // Session budget should be a valid JSON object with budget fields
    assert!(
        body.is_object(),
        "GET /budget/{{session_id}} should return a JSON object, got: {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /budget (global budget)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_budget_global() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/budget").await;

    assert!(
        body.is_object(),
        "GET /budget should return a JSON object, got: {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /calibration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_calibration() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/calibration").await;

    // Should contain brier_score, sample_count, buckets
    assert!(
        body.get("brier_score").is_some(),
        "GET /calibration should contain 'brier_score', got: {body:?}"
    );
    assert!(
        body.get("sample_count").is_some(),
        "GET /calibration should contain 'sample_count', got: {body:?}"
    );
    assert!(
        body.get("buckets").and_then(|v| v.as_array()).is_some(),
        "GET /calibration should contain 'buckets' array, got: {body:?}"
    );

    // Verify buckets structure: 5 buckets with expected fields
    let buckets = body["buckets"].as_array().unwrap();
    assert_eq!(
        buckets.len(),
        5,
        "calibration should have 5 buckets, got {}",
        buckets.len()
    );
    for bucket in buckets {
        assert!(
            bucket.get("range").is_some(),
            "each bucket should have 'range'"
        );
        assert!(
            bucket.get("count").is_some(),
            "each bucket should have 'count'"
        );
        assert!(
            bucket.get("avg_predicted").is_some(),
            "each bucket should have 'avg_predicted'"
        );
        assert!(
            bucket.get("avg_actual").is_some(),
            "each bucket should have 'avg_actual'"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: GET /memory/stats
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_memory_stats() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/memory/stats").await;

    assert!(
        body.get("event_count").is_some(),
        "GET /memory/stats should contain 'event_count', got: {body:?}"
    );
    // Fresh in-memory DB should start at 0
    let count = body["event_count"].as_u64().unwrap_or(u64::MAX);
    assert_eq!(
        count, 0,
        "fresh daemon should have 0 events, got {count}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /memory/graph/stats
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_graph_stats() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/memory/graph/stats").await;

    assert!(
        body.get("node_count").is_some(),
        "GET /memory/graph/stats should contain 'node_count', got: {body:?}"
    );
    assert!(
        body.get("edge_count").is_some(),
        "GET /memory/graph/stats should contain 'edge_count', got: {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /profiler/fatigue
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_profiler_fatigue() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/profiler/fatigue").await;

    assert!(
        body.get("score").is_some(),
        "GET /profiler/fatigue should contain 'score', got: {body:?}"
    );
    assert!(
        body.get("high_friction").is_some(),
        "GET /profiler/fatigue should contain 'high_friction', got: {body:?}"
    );
    assert!(
        body.get("signals").is_some(),
        "GET /profiler/fatigue should contain 'signals', got: {body:?}"
    );
    assert!(
        body.get("recommendation").is_some(),
        "GET /profiler/fatigue should contain 'recommendation', got: {body:?}"
    );

    // Initial fatigue score should be 0
    let score = body["score"].as_f64().unwrap_or(-1.0);
    assert!(
        score >= 0.0,
        "initial fatigue score should be non-negative, got {score}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /heuristics/population
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_heuristics_population() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/heuristics/population").await;

    assert!(
        body.get("size").is_some(),
        "GET /heuristics/population should contain 'size', got: {body:?}"
    );
    assert!(
        body.get("generation").is_some(),
        "GET /heuristics/population should contain 'generation', got: {body:?}"
    );
    assert!(
        body.get("best_fitness").is_some(),
        "GET /heuristics/population should contain 'best_fitness', got: {body:?}"
    );

    // The evolver is seeded with at least 1 member
    let size = body["size"].as_u64().unwrap_or(0);
    assert!(
        size >= 1,
        "population should be seeded with at least 1 member, got {size}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /heuristics/best
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_heuristics_best() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/heuristics/best").await;

    // The evolver is seeded, so 'best' key should be present
    assert!(
        body.get("best").is_some(),
        "GET /heuristics/best should contain 'best', got: {body:?}"
    );

    let best = &body["best"];
    assert!(
        best.get("id").is_some(),
        "best heuristic should have 'id', got: {best:?}"
    );
    assert!(
        best.get("generation").is_some(),
        "best heuristic should have 'generation', got: {best:?}"
    );
    assert!(
        best.get("fitness").is_some(),
        "best heuristic should have 'fitness', got: {best:?}"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /replay/sessions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_replay_sessions_empty() {
    let (client, base) = start_test_daemon().await;

    let body = get_json(&client, &base, "/replay/sessions").await;

    assert!(
        body.get("sessions").and_then(|v| v.as_array()).is_some(),
        "GET /replay/sessions should contain 'sessions' array, got: {body:?}"
    );

    // Fresh daemon should have no replay sessions
    let sessions = body["sessions"].as_array().unwrap();
    assert!(
        sessions.is_empty(),
        "fresh daemon should have 0 replay sessions, got {}",
        sessions.len()
    );
}

// ---------------------------------------------------------------------------
// Test: GET /replay/sessions after a session is recorded
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_replay_sessions_after_hooks() {
    let (client, base) = start_test_daemon().await;

    // Run hooks to generate replay data
    post_json(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "replay-coverage-test",
            "prompt": "test prompt for replay"
        }),
    )
    .await;

    post_json(
        &client,
        &base,
        "/hooks/stop",
        json!({
            "hook_event_name": "Stop",
            "session_id": "replay-coverage-test",
            "last_assistant_message": "Done."
        }),
    )
    .await;

    // Wait for async post-processing
    tokio::time::sleep(Duration::from_millis(500)).await;

    let body = get_json(&client, &base, "/replay/sessions").await;
    let sessions = body["sessions"]
        .as_array()
        .expect("sessions should be an array");

    assert!(
        !sessions.is_empty(),
        "replay should have recorded sessions after hooks"
    );

    // Verify session entry structure
    let session = &sessions[0];
    assert!(
        session.get("session_id").is_some(),
        "session entry should have 'session_id'"
    );
    assert!(
        session.get("event_count").is_some(),
        "session entry should have 'event_count'"
    );
}

// ---------------------------------------------------------------------------
// Test: GET /replay/{session_id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_replay_session_detail() {
    let (client, base) = start_test_daemon().await;

    // Create session with hooks
    post_json(
        &client,
        &base,
        "/hooks/user-prompt-submit",
        json!({
            "hook_event_name": "UserPromptSubmit",
            "session_id": "replay-detail-test",
            "prompt": "test"
        }),
    )
    .await;

    // Wait for replay recording
    tokio::time::sleep(Duration::from_millis(300)).await;

    let body = get_json(&client, &base, "/replay/replay-detail-test").await;

    assert!(
        body.get("session_id").is_some(),
        "GET /replay/{{session_id}} should contain 'session_id', got: {body:?}"
    );
    assert!(
        body.get("events").and_then(|v| v.as_array()).is_some(),
        "GET /replay/{{session_id}} should contain 'events' array, got: {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Test: POST /budget/consume
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_budget_consume() {
    let (client, base) = start_test_daemon().await;

    let body = post_json(
        &client,
        &base,
        "/budget/consume",
        json!({
            "tokens": 100,
            "cost_usd": 0.01
        }),
    )
    .await;

    assert!(
        body.get("summary").is_some(),
        "POST /budget/consume should return 'summary', got: {body:?}"
    );
    assert!(
        body.get("is_over_budget").is_some(),
        "POST /budget/consume should return 'is_over_budget', got: {body:?}"
    );
    assert!(
        body.get("consumed_tokens").is_some(),
        "POST /budget/consume should return 'consumed_tokens', got: {body:?}"
    );

    let consumed = body["consumed_tokens"].as_u64().unwrap_or(0);
    assert_eq!(
        consumed, 100,
        "consumed_tokens should be 100 after consuming 100, got {consumed}"
    );
}

// ---------------------------------------------------------------------------
// Test: POST /profiler/signal
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_profiler_signal() {
    let (client, base) = start_test_daemon().await;

    // Record 3 consecutive error signals to trigger ErrorLoop fatigue signal
    // (on_error only pushes a signal when consecutive_errors >= 3)
    let mut last_signal_resp = serde_json::Value::Null;
    for _ in 0..3 {
        last_signal_resp = post_json(
            &client,
            &base,
            "/profiler/signal",
            json!({ "signal_type": "error" }),
        )
        .await;
    }

    // POST /profiler/signal should itself return a FatigueReport
    assert!(
        last_signal_resp.get("score").is_some(),
        "POST /profiler/signal should return 'score', got: {last_signal_resp:?}"
    );

    // Get the fatigue report after error signals
    let body = get_json(&client, &base, "/profiler/fatigue").await;

    // Should return a FatigueReport
    assert!(
        body.get("score").is_some(),
        "GET /profiler/fatigue should return 'score', got: {body:?}"
    );
    assert!(
        body.get("high_friction").is_some(),
        "GET /profiler/fatigue should return 'high_friction', got: {body:?}"
    );

    // 3 consecutive errors should trigger ErrorLoop and increase score
    let score = body["score"].as_f64().unwrap_or(0.0);
    assert!(
        score > 0.0,
        "fatigue score should be > 0 after 3 error signals, got {score}"
    );
}

// ---------------------------------------------------------------------------
// Test: POST /memory/graph/search (empty graph)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_graph_search_empty() {
    let (client, base) = start_test_daemon().await;

    let body = post_json(
        &client,
        &base,
        "/memory/graph/search",
        json!({ "query": "anything", "limit": 5 }),
    )
    .await;

    assert!(
        body.get("nodes").and_then(|v| v.as_array()).is_some(),
        "POST /memory/graph/search should return 'nodes' array, got: {body:?}"
    );

    let nodes = body["nodes"].as_array().unwrap();
    assert!(
        nodes.is_empty(),
        "fresh daemon graph search should return 0 nodes, got {}",
        nodes.len()
    );
}

// ---------------------------------------------------------------------------
// Test: POST /memory/recall (empty memory)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn endpoint_memory_recall_empty() {
    let (client, base) = start_test_daemon().await;

    let body = post_json(
        &client,
        &base,
        "/memory/recall",
        json!({ "query": "anything", "limit": 5 }),
    )
    .await;

    assert!(
        body.get("events").and_then(|v| v.as_array()).is_some(),
        "POST /memory/recall should return 'events' array, got: {body:?}"
    );
}
