use std::path::PathBuf;

use anyhow::Result;
use tokio::sync::watch;
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

    // Create a shutdown watch channel for the /admin/shutdown endpoint
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let app = metaygn_daemon::build_app_with_state(state)
        .layer(axum::Extension(shutdown_tx));

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
        .with_graceful_shutdown(shutdown_signal(port_file.clone(), shutdown_rx))
        .await?;

    // Cleanup port file after shutdown (in case shutdown_signal didn't remove it)
    if port_file.exists() {
        let _ = std::fs::remove_file(&port_file);
        tracing::info!("Removed port file during final cleanup");
    }

    Ok(())
}

/// Wait for Ctrl+C or the shutdown watch channel, then clean up the port file.
async fn shutdown_signal(port_file: PathBuf, mut shutdown_rx: watch::Receiver<bool>) {
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

    // Remove port file on clean shutdown
    if port_file.exists() {
        let _ = std::fs::remove_file(&port_file);
        tracing::info!("Removed port file");
    }
}
