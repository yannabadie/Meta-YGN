use anyhow::{Context, Result};
use serde_json::Value;

use crate::util::{http_client, read_daemon_port};

/// Status command: query /health and display formatted status.
pub async fn cmd_status() -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon:  STOPPED");
        println!("  (no port file at ~/.claude/aletheia/daemon.port)");
        return Ok(());
    };

    let client = http_client()?;
    let url = format!("http://127.0.0.1:{port}/health");

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp
                .json()
                .await
                .context("failed to parse health response")?;

            let status = body
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let version = body
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let kernel = body
                .get("kernel")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            println!("Daemon:  RUNNING");
            println!("  Port:    {port}");
            println!("  Status:  {status}");
            println!("  Version: {version}");
            println!("  Kernel:  {kernel}");
        }
        Ok(resp) => {
            println!("Daemon:  ERROR");
            println!("  Port:    {port}");
            println!("  HTTP:    {}", resp.status());
        }
        Err(_) => {
            println!("Daemon:  STOPPED");
            println!("  (port file found with port {port}, but daemon is not responding)");
        }
    }

    Ok(())
}
