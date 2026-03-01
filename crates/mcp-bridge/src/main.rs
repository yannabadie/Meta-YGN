use anyhow::Context;
use metaygn_mcp_bridge::handler::AletheiaHandler;
use metaygn_mcp_bridge::{DaemonClient, read_daemon_port};
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Send tracing output to stderr (stdout is reserved for MCP stdio transport).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("aletheia-mcp starting...");

    let port = read_daemon_port().context("daemon not running — no port file found")?;
    let daemon = DaemonClient::new(port)?;
    let handler = AletheiaHandler::new(daemon);

    let service = handler.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
