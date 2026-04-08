use anyhow::Result;
use serde_json::Value;

use crate::util::{http_client, read_daemon_port};

/// Replay command: view session hook timelines.
pub async fn cmd_replay(session_id: Option<&str>) -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running (no port file found).");
        return Ok(());
    };
    let client = http_client()?;

    match session_id {
        None => {
            let url = format!("http://127.0.0.1:{port}/replay/sessions");
            let resp = client.get(&url).send().await?;
            let body: Value = resp.json().await?;
            let sessions = body
                .get("sessions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if sessions.is_empty() {
                println!("No replay sessions recorded yet.");
                return Ok(());
            }
            println!(
                "{:<40} {:>6}  {:<20}  {:<20}",
                "SESSION", "EVENTS", "FIRST", "LAST"
            );
            println!("{}", "-".repeat(90));
            for s in &sessions {
                let sid = s.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
                let count = s.get("event_count").and_then(|v| v.as_u64()).unwrap_or(0);
                let first = s.get("first_event").and_then(|v| v.as_str()).unwrap_or("?");
                let last = s.get("last_event").and_then(|v| v.as_str()).unwrap_or("?");
                println!("{:<40} {:>6}  {:<20}  {:<20}", sid, count, first, last);
            }
        }
        Some(sid) => {
            let url = format!("http://127.0.0.1:{port}/replay/{sid}");
            let resp = client.get(&url).send().await?;
            let body: Value = resp.json().await?;
            let events = body
                .get("events")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if events.is_empty() {
                println!("No events found for session: {sid}");
                return Ok(());
            }
            println!("Session: {sid}");
            println!("Events: {}\n", events.len());
            for (i, event) in events.iter().enumerate() {
                let hook = event
                    .get("hook_event")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let latency = event
                    .get("latency_ms")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let timestamp = event
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                println!("[{:>3}] {} ({latency}ms) @ {timestamp}", i + 1, hook);
                if let Some(req) = event.get("request") {
                    let summary = serde_json::to_string(req).unwrap_or_default();
                    if summary.len() > 120 {
                        println!("      req: {}...", &summary[..120]);
                    } else {
                        println!("      req: {summary}");
                    }
                }
                if let Some(resp) = event.get("response") {
                    let summary = serde_json::to_string(resp).unwrap_or_default();
                    if summary.len() > 120 {
                        println!("      res: {}...", &summary[..120]);
                    } else {
                        println!("      res: {summary}");
                    }
                }
                println!();
            }
        }
    }
    Ok(())
}
