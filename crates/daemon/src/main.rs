use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tokio::sync::watch;

use metaygn_daemon::app_state::AppState;
use metaygn_daemon::auth::AuthToken;

mod telemetry;

#[derive(Parser)]
#[command(name = "aletheiad", about = "Aletheia metacognitive daemon")]
struct Args {
    /// Run as MCP stdio server instead of HTTP daemon
    #[arg(long)]
    mcp: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    telemetry::init_tracing(args.mcp)
        .expect("failed to initialise tracing");

    // Determine database path:
    // 1. METAYGN_DB_PATH env var (set by CLI --db-path flag)
    // 2. Default: ~/.claude/aletheia/metaygn.db
    let db_path = std::env::var("METAYGN_DB_PATH")
        .ok()
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("could not determine home directory")
                .join(".claude")
                .join("aletheia")
                .join("metaygn.db")
        });

    let db_dir = db_path
        .parent()
        .expect("db_path has no parent directory")
        .to_path_buf();
    std::fs::create_dir_all(&db_dir)?;

    tracing::info!("Opening database at {}", db_path.display());
    let state = AppState::new(
        db_path
            .to_str()
            .expect("database path contains invalid UTF-8"),
    )
    .await?;

    if args.mcp {
        run_mcp_server(state).await
    } else {
        run_http_server(state, db_dir).await
    }
}

/// Run the daemon as an MCP stdio server (--mcp flag).
async fn run_mcp_server(state: AppState) -> Result<()> {
    #[cfg(feature = "mcp")]
    {
        use rmcp::ServiceExt;
        tracing::info!("aletheiad starting in MCP stdio mode...");
        let handler = metaygn_daemon::mcp::mcp_handler::AletheiaHandler::new(state);
        let service = handler.serve(rmcp::transport::io::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }
    #[cfg(not(feature = "mcp"))]
    {
        let _ = state; // suppress unused warning
        anyhow::bail!(
            "MCP feature not enabled. Rebuild with: cargo build -p metaygn-daemon --features mcp"
        );
    }
}

/// Run the daemon as an HTTP server (default mode).
async fn run_http_server(state: AppState, db_dir: PathBuf) -> Result<()> {
    // Create a shutdown watch channel for the /admin/shutdown endpoint
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Generate a random auth token and write it next to the port file.
    let token_string = uuid::Uuid::new_v4().to_string();
    let token = AuthToken(token_string.clone());
    let token_file = db_dir.join("daemon.token");
    std::fs::write(&token_file, &token_string)?;
    tracing::info!("Token file written to {}", token_file.display());

    let app = metaygn_daemon::build_app_with_state(state, token).layer(axum::Extension(shutdown_tx));

    // Bind to dynamic port on localhost
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tracing::info!("aletheiad listening on {addr}");

    // Write port file so clients can discover us
    let port_file = db_dir.join("daemon.port");
    std::fs::write(&port_file, addr.port().to_string())?;
    tracing::info!("Port file written to {}", port_file.display());

    // Serve with graceful shutdown on Ctrl+C or /admin/shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(port_file.clone(), token_file.clone(), shutdown_rx))
        .await?;

    // Cleanup port and token files after shutdown (in case shutdown_signal didn't remove them)
    for f in [&port_file, &token_file] {
        if f.exists() {
            let _ = std::fs::remove_file(f);
            tracing::info!("Removed {} during final cleanup", f.display());
        }
    }

    Ok(())
}

/// Wait for Ctrl+C or the shutdown watch channel, then clean up the port and token files.
async fn shutdown_signal(
    port_file: PathBuf,
    token_file: PathBuf,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl+C received, shutting down...");
        }
        _ = async {
            while !*shutdown_rx.borrow_and_update() {
                if shutdown_rx.changed().await.is_err() {
                    // Sender dropped -- shut down
                    break;
                }
            }
        } => {
            tracing::info!("/admin/shutdown requested, shutting down...");
        }
    }

    // Remove port and token files on clean shutdown
    for f in [&port_file, &token_file] {
        if f.exists() {
            let _ = std::fs::remove_file(f);
            tracing::info!("Removed {}", f.display());
        }
    }
}
