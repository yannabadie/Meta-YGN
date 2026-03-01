mod tui;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

/// Port file location: ~/.claude/aletheia/daemon.port
fn port_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".claude").join("aletheia").join("daemon.port"))
}

/// Read the daemon port from the port file.
/// Returns None if the file doesn't exist or can't be parsed.
fn read_daemon_port() -> Option<u16> {
    let path = port_file_path().ok()?;
    let contents = std::fs::read_to_string(path).ok()?;
    contents.trim().parse::<u16>().ok()
}

/// Build a reqwest client with a 2-second timeout.
fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("failed to build HTTP client")
}

#[derive(Parser)]
#[command(name = "aletheia", about = "MetaYGN metacognitive runtime CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the aletheia daemon
    Start {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to bind to (0 = auto)
        #[arg(long, default_value_t = 0)]
        port: u16,

        /// Path to the SQLite database
        #[arg(long)]
        db_path: Option<PathBuf>,
    },

    /// Stop the running daemon
    Stop,

    /// Show daemon status
    Status,

    /// Recall memories from the daemon
    Recall {
        /// Search query
        #[arg(long)]
        query: String,

        /// Maximum number of results
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },

    /// Launch real-time cognitive telemetry dashboard
    Top,

    /// Initialize MetaYGN configuration in current project
    Init {
        /// Overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Replay a past session's hook timeline
    Replay {
        /// Session ID to replay (omit to list sessions)
        session_id: Option<String>,
    },

    /// Export RL trajectories to JSONL file
    Export {
        /// Maximum number of trajectories to export
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },

    /// Launch MCP stdio server (for Claude Code / MCP clients)
    Mcp,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            host,
            port,
            db_path,
        } => cmd_start(&host, port, db_path.as_deref()).await,
        Commands::Stop => cmd_stop().await,
        Commands::Status => cmd_status().await,
        Commands::Recall { query, limit } => cmd_recall(&query, limit).await,
        Commands::Top => cmd_top().await,
        Commands::Init { force } => cmd_init(force),
        Commands::Replay { session_id } => cmd_replay(session_id.as_deref()).await,
        Commands::Export { limit } => cmd_export(limit).await,
        Commands::Mcp => cmd_mcp().await,
    }
}

/// Start command: spawn the aletheiad daemon as a detached process.
async fn cmd_start(_host: &str, _port: u16, db_path: Option<&std::path::Path>) -> Result<()> {
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

/// Stop command: POST /admin/shutdown and wait for port file removal.
async fn cmd_stop() -> Result<()> {
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

/// Status command: query /health and display formatted status.
async fn cmd_status() -> Result<()> {
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

/// Recall command: POST /memory/recall and pretty-print results.
async fn cmd_recall(query: &str, limit: u32) -> Result<()> {
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

/// Init command: create .claude/settings.json for MetaYGN project onboarding.
fn cmd_init(force: bool) -> Result<()> {
    let config_dir = std::path::Path::new(".claude");
    let settings_path = config_dir.join("settings.json");

    if settings_path.exists() && !force {
        println!("Configuration already exists at .claude/settings.json");
        println!("Use --force to overwrite.");
        return Ok(());
    }

    std::fs::create_dir_all(config_dir)?;

    let settings = serde_json::json!({
        "enabledPlugins": {
            "aletheia-nexus@local": true
        },
        "outputStyle": "aletheia-proof"
    });

    std::fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    println!("MetaYGN initialized!");
    println!("  Created: .claude/settings.json");
    println!();
    println!("Next steps:");
    println!("  1. Start the daemon:  aletheia start (or cargo run -p metaygn-daemon)");
    println!("  2. Use Claude Code:   claude --plugin-dir /path/to/MetaYGN");
    println!("  3. Check status:      aletheia status");

    Ok(())
}

/// Replay command: view session hook timelines.
async fn cmd_replay(session_id: Option<&str>) -> Result<()> {
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

/// Export command: fetch RL trajectories from daemon and write to a JSONL file.
async fn cmd_export(limit: u32) -> Result<()> {
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

/// Mcp command: launch the MCP stdio bridge (aletheia-mcp) with inherited I/O.
async fn cmd_mcp() -> Result<()> {
    let exe = std::env::current_exe().context("could not determine own executable path")?;
    let exe_dir = exe.parent().context("executable has no parent directory")?;
    let mcp_name = if cfg!(windows) {
        "aletheia-mcp.exe"
    } else {
        "aletheia-mcp"
    };
    let mcp_path = exe_dir.join(mcp_name);

    if !mcp_path.exists() {
        anyhow::bail!(
            "Cannot find aletheia-mcp at {:?}. Build with: cargo build --workspace",
            mcp_path
        );
    }

    let status = std::process::Command::new(&mcp_path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to launch aletheia-mcp")?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Top command: launch the real-time cognitive telemetry TUI dashboard.
async fn cmd_top() -> Result<()> {
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
            // Daemon is up — launch TUI
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
