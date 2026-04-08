use anyhow::{Context, Result};
use serde_json::Value;

use crate::util::{http_client, read_daemon_port};

/// Recall command: POST /memory/recall and pretty-print results.
pub async fn cmd_recall(query: &str, limit: u32) -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running (no port file found).");
        println!("Start the daemon first with: aletheia start");
        return Ok(());
    };

    let client = http_client()?;
    let url = format!("http://127.0.0.1:{port}/memory/recall");

    let body = serde_json::json!({
        "query": query,
        "limit": limit,
    });

    let resp = match client.post(&url).json(&body).send().await {
        Ok(r) => r,
        Err(_) => {
            println!("Daemon not running (could not connect on port {port}).");
            return Ok(());
        }
    };

    if !resp.status().is_success() {
        println!("Error: daemon returned HTTP {}", resp.status());
        return Ok(());
    }

    let result: Value = resp
        .json()
        .await
        .context("failed to parse recall response")?;

    // Check for error in response
    if let Some(err) = result.get("error").and_then(|v| v.as_str()) {
        println!("Error from daemon: {err}");
        return Ok(());
    }

    // Pretty-print the events
    let events = result
        .get("events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if events.is_empty() {
        println!("No memories found for query: \"{query}\"");
        return Ok(());
    }

    println!(
        "Found {} memory event(s) for query: \"{query}\"\n",
        events.len()
    );

    for (i, event) in events.iter().enumerate() {
        let event_type = event
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let session_id = event
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let timestamp = event
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let payload = event
            .get("payload")
            .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()))
            .unwrap_or_default();

        println!("--- Event {} ---", i + 1);
        println!("  Type:      {event_type}");
        println!("  Session:   {session_id}");
        println!("  Timestamp: {timestamp}");
        if !payload.is_empty() {
            println!("  Payload:   {payload}");
        }
        println!();
    }

    Ok(())
}
