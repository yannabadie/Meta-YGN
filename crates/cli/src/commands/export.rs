use anyhow::{Context, Result};

use crate::util::{http_client, read_daemon_port};

/// Export command: fetch RL trajectories from daemon and write to a JSONL file.
pub async fn cmd_export(limit: u32) -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running (no port file found).");
        return Ok(());
    };

    let client = http_client()?;
    let url = format!("http://127.0.0.1:{port}/trajectories/export?limit={limit}");

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => {
            println!("Cannot connect to daemon on port {port}.");
            return Ok(());
        }
    };

    let body: serde_json::Value = resp
        .json()
        .await
        .context("failed to parse export response")?;

    let trajectories = body
        .get("trajectories")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if trajectories.is_empty() {
        println!("No trajectories to export.");
        return Ok(());
    }

    // Write to ~/.claude/aletheia/trajectories/
    let home = dirs::home_dir().context("could not determine home directory")?;
    let export_dir = home.join(".claude").join("aletheia").join("trajectories");
    std::fs::create_dir_all(&export_dir)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("export-{timestamp}.jsonl");
    let filepath = export_dir.join(&filename);

    let mut output = String::new();
    for t in &trajectories {
        output.push_str(&serde_json::to_string(t).unwrap_or_default());
        output.push('\n');
    }
    std::fs::write(&filepath, &output)?;

    println!(
        "Exported {} trajectories to {}",
        trajectories.len(),
        filepath.display()
    );
    Ok(())
}
