mod commands;
mod tui;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    /// Run calibration evaluation on session data
    Eval,

    /// Check MetaYGN installation health
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { db_path } => commands::cmd_start(db_path.as_deref()).await,
        Commands::Stop => commands::cmd_stop().await,
        Commands::Status => commands::cmd_status().await,
        Commands::Recall { query, limit } => commands::cmd_recall(&query, limit).await,
        Commands::Top => commands::cmd_top().await,
        Commands::Init { force } => commands::cmd_init(force),
        Commands::Replay { session_id } => commands::cmd_replay(session_id.as_deref()).await,
        Commands::Export { limit } => commands::cmd_export(limit).await,
        Commands::Mcp => commands::cmd_mcp().await,
        Commands::Eval => commands::cmd_eval().await,
        Commands::Doctor => commands::cmd_doctor().await,
    }
}
