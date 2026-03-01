use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Send tracing output to stderr (stdout is reserved for MCP stdio transport).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("aletheia-mcp starting...");
}
