use anyhow::Result;

use crate::tui;
use crate::util::{http_client, read_daemon_port};

/// Top command: launch the real-time cognitive telemetry TUI dashboard.
pub async fn cmd_top() -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon not running (no port file found).");
        println!("Start the daemon first, then run: aletheia top");
        return Ok(());
    };

    // Quick health check before launching the TUI
    let client = http_client()?;
    let url = format!("http://127.0.0.1:{port}/health");

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Daemon is up -- launch TUI
            tui::run_tui(port).await
        }
        Ok(resp) => {
            println!(
                "Daemon returned unexpected status {} on port {port}.",
                resp.status()
            );
            println!("Fix the daemon and try again.");
            Ok(())
        }
        Err(_) => {
            println!("Cannot connect to daemon on port {port}.");
            println!("Start the daemon first, then run: aletheia top");
            Ok(())
        }
    }
}
