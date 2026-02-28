use std::path::PathBuf;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

use metaygn_daemon::app_state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Determine database path: ~/.claude/aletheia/metaygn.db
    let db_dir = dirs::home_dir()
        .expect("could not determine home directory")
        .join(".claude")
        .join("aletheia");
    std::fs::create_dir_all(&db_dir)?;
    let db_path = db_dir.join("metaygn.db");

    tracing::info!("Opening database at {}", db_path.display());
    let state = AppState::new(db_path.to_str().unwrap()).await?;
    let app = metaygn_daemon::build_app_with_state(state);

    // Bind to dynamic port on localhost
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tracing::info!("aletheiad listening on {addr}");

    // Write port file so clients can discover us
    let port_file = db_dir.join("daemon.port");
    std::fs::write(&port_file, addr.port().to_string())?;
    tracing::info!("Port file written to {}", port_file.display());

    // Serve with graceful shutdown on Ctrl+C
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(port_file))
        .await?;

    Ok(())
}

/// Wait for Ctrl+C (SIGTERM on Unix), then clean up the port file.
async fn shutdown_signal(port_file: PathBuf) {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for Ctrl+C");
    tracing::info!("Shutdown signal received, cleaning up...");

    // Remove port file on clean shutdown
    if port_file.exists() {
        let _ = std::fs::remove_file(&port_file);
        tracing::info!("Removed port file");
    }
}
