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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { host, port, db_path } => cmd_start(&host, port, db_path.as_deref()),
        Commands::Stop => cmd_stop().await,
        Commands::Status => cmd_status().await,
        Commands::Recall { query, limit } => cmd_recall(&query, limit).await,
    }
}

/// Start command: prints a message (actual daemon spawn deferred to Phase 2).
fn cmd_start(host: &str, port: u16, db_path: Option<&std::path::Path>) -> Result<()> {
    let port_display = if port == 0 {
        "auto".to_string()
    } else {
        port.to_string()
    };
    let db_display = db_path
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "~/.claude/aletheia/metaygn.db".to_string());

    println!("Starting aletheia daemon...");
    println!("  Host:    {host}");
    println!("  Port:    {port_display}");
    println!("  DB:      {db_display}");
    println!();
    println!("NOTE: Daemon spawn not yet implemented (Phase 2).");
    println!("      Run `aletheiad` directly for now.");
    Ok(())
}

/// Stop command: check if daemon is running and report status.
async fn cmd_stop() -> Result<()> {
    let Some(port) = read_daemon_port() else {
        println!("Daemon: STOPPED (no port file found)");
        return Ok(());
    };

    // Check if the daemon is actually reachable
    let client = http_client()?;
    let url = format!("http://127.0.0.1:{port}/health");

    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("Daemon is running on port {port}.");
            println!("NOTE: Graceful stop not yet implemented (Phase 2).");
            println!("      Send Ctrl+C to the daemon process for now.");
        }
        _ => {
            println!("Daemon: STOPPED (port file exists but daemon is not responding)");
            // Clean up stale port file
            if let Ok(path) = port_file_path() {
                let _ = std::fs::remove_file(path);
                println!("  Removed stale port file.");
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
            let body: Value = resp.json().await.context("failed to parse health response")?;

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

    let result: Value = resp.json().await.context("failed to parse recall response")?;

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

    println!("Found {} memory event(s) for query: \"{query}\"\n", events.len());

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
            .map(|v| {
                serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
            })
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
