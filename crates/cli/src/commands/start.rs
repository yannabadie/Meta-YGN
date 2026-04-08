use std::time::Duration;

use anyhow::{Context, Result};

use crate::util::{http_client, port_file_path, read_daemon_port};

/// Start command: spawn the aletheiad daemon as a detached process.
pub async fn cmd_start(db_path: Option<&std::path::Path>) -> Result<()> {
    // 1. Check if already running
    if let Some(existing_port) = read_daemon_port() {
        let client = http_client()?;
        if let Ok(resp) = client
            .get(format!("http://127.0.0.1:{existing_port}/health"))
            .send()
            .await
            && resp.status().is_success()
        {
            println!("Daemon already running on port {existing_port}");
            return Ok(());
        }
        // Port file exists but daemon not responding -- stale
        if let Ok(pf) = port_file_path() {
            let _ = std::fs::remove_file(&pf);
        }
    }

    // 2. Find aletheiad binary next to this executable
    let exe = std::env::current_exe().context("could not determine own executable path")?;
    let exe_dir = exe.parent().context("executable has no parent directory")?;
    let daemon_name = if cfg!(windows) {
        "aletheiad.exe"
    } else {
        "aletheiad"
    };
    let daemon_path = exe_dir.join(daemon_name);

    if !daemon_path.exists() {
        anyhow::bail!(
            "Cannot find aletheiad at {:?}. Build with: cargo build --workspace",
            daemon_path
        );
    }

    // 3. Spawn detached
    println!("Starting aletheiad...");
    let mut cmd = std::process::Command::new(&daemon_path);
    if let Some(db) = db_path {
        cmd.env("METAYGN_DB_PATH", db);
    }
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null());
    cmd.spawn().context("Failed to spawn aletheiad")?;

    // 4. Poll for port file (up to 10 seconds)
    let pf = port_file_path()?;
    let mut found = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        if pf.exists() {
            found = true;
            break;
        }
    }

    if !found {
        anyhow::bail!("Daemon did not start within 10 seconds");
    }

    let port = read_daemon_port().context("Port file exists but unreadable")?;

    // 5. Health check
    let client = http_client()?;
    match client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            println!("Daemon started on port {port}");
        }
        _ => {
            println!(
                "Daemon started on port {port} (health check failed -- may still be initializing)"
            );
        }
    }

    Ok(())
}
