use std::time::Duration;

use anyhow::Result;

use crate::util::{http_client, port_file_path, read_daemon_port};

/// Stop command: POST /admin/shutdown and wait for port file removal.
pub async fn cmd_stop() -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon: STOPPED (no port file found)");
        return Ok(());
    };

    let client = http_client()?;
    println!("Stopping daemon on port {port}...");

    // Send shutdown request
    match client
        .post(format!("http://127.0.0.1:{port}/admin/shutdown"))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            // Poll until port file disappears (up to 5 seconds)
            let pf = port_file_path()?;
            let mut stopped = false;
            for _ in 0..10 {
                tokio::time::sleep(Duration::from_millis(500)).await;
                if !pf.exists() {
                    stopped = true;
                    break;
                }
            }
            if stopped {
                println!("Daemon stopped.");
            } else {
                // Force cleanup
                let _ = std::fs::remove_file(&pf);
                println!("Daemon stopped (port file cleaned up).");
            }
        }
        _ => {
            println!("Daemon not responding. Cleaning up port file.");
            if let Ok(pf) = port_file_path() {
                let _ = std::fs::remove_file(&pf);
            }
        }
    }

    Ok(())
}
