use anyhow::Result;
use serde_json::Value;

use crate::util::{http_client, read_daemon_port};

/// Eval command: compute and display calibration metrics from daemon data.
pub async fn cmd_eval() -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running. Start with: aletheia start");
        return Ok(());
    };

    let client = http_client()?;
    let base = format!("http://127.0.0.1:{port}");

    // 1. Get heuristic outcomes
    let resp = client
        .get(format!("{base}/heuristics/population"))
        .send()
        .await?;
    let _pop: Value = resp.json().await?;

    // 2. Get session replay data
    let resp = client.get(format!("{base}/replay/sessions")).send().await?;
    let sessions: Value = resp.json().await?;

    // 3. Get memory stats
    let resp = client.get(format!("{base}/memory/stats")).send().await?;
    let memory: Value = resp.json().await?;

    // 4. Get graph stats
    let resp = client
        .get(format!("{base}/memory/graph/stats"))
        .send()
        .await?;
    let graph: Value = resp.json().await?;

    // Compute metrics
    let session_count = sessions
        .get("sessions")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let event_count = memory
        .get("event_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let node_count = graph
        .get("node_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let edge_count = graph
        .get("edge_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Display report
    println!("\u{2554}{}\u{2557}", "\u{2550}".repeat(46));
    println!("\u{2551}     Aletheia Calibration Report              \u{2551}");
    println!("\u{2560}{}\u{2563}", "\u{2550}".repeat(46));
    println!(
        "\u{2551}  Sessions recorded:  {:>6}                  \u{2551}",
        session_count
    );
    println!(
        "\u{2551}  Events logged:      {:>6}                  \u{2551}",
        event_count
    );
    println!(
        "\u{2551}  Graph nodes:        {:>6}                  \u{2551}",
        node_count
    );
    println!(
        "\u{2551}  Graph edges:        {:>6}                  \u{2551}",
        edge_count
    );
    println!("\u{2560}{}\u{2563}", "\u{2550}".repeat(46));

    if session_count < 5 {
        println!("\u{2551}  Insufficient data for calibration metrics   \u{2551}");
        println!("\u{2551}  (need at least 5 sessions)                  \u{2551}");
    } else {
        // Get best heuristic fitness
        let resp = client.get(format!("{base}/heuristics/best")).send().await?;
        let best: Value = resp.json().await?;
        let fitness = best
            .get("fitness")
            .and_then(|f| f.get("composite"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let success_rate = best
            .get("fitness")
            .and_then(|f| f.get("verification_success_rate"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        println!(
            "\u{2551}  Best heuristic fitness:  {:.3}              \u{2551}",
            fitness
        );
        println!(
            "\u{2551}  Verification success:    {:.1}%              \u{2551}",
            success_rate * 100.0
        );
    }

    // Get fatigue
    let resp = client
        .get(format!("{base}/profiler/fatigue"))
        .send()
        .await?;
    let fatigue: Value = resp.json().await?;
    let fatigue_score = fatigue.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
    println!(
        "\u{2551}  Current fatigue:       {:.2}                \u{2551}",
        fatigue_score
    );

    // Calibration / Brier score
    if let Ok(resp) = client.get(format!("{base}/calibration")).send().await
        && let Ok(cal) = resp.json::<Value>().await
    {
        let brier = cal
            .get("brier_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let count = cal
            .get("sample_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        println!("\u{2560}{}\u{2563}", "\u{2550}".repeat(46));
        println!(
            "\u{2551}  Brier score:         {:.4}  (n={:<5})      \u{2551}",
            brier, count
        );
        if let Some(buckets) = cal.get("buckets").and_then(|v| v.as_array()) {
            for b in buckets {
                let range = b.get("range").and_then(|v| v.as_str()).unwrap_or("?");
                let predicted = b
                    .get("avg_predicted")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let actual = b.get("avg_actual").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let n = b.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                if n > 0 {
                    println!(
                        "\u{2551}    {:>8}: pred {:.0}% actual {:.0}% (n={:<3})\u{2551}",
                        range,
                        predicted * 100.0,
                        actual * 100.0,
                        n
                    );
                }
            }
        }
    }

    println!("\u{255a}{}\u{255d}", "\u{2550}".repeat(46));

    Ok(())
}
