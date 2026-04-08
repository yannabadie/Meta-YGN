use anyhow::{Context, Result};

/// Mcp command: launch the daemon in MCP stdio mode (aletheiad --mcp).
pub async fn cmd_mcp() -> Result<()> {
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
            "Cannot find aletheiad at {:?}. Build with: cargo build --workspace --features mcp",
            daemon_path
        );
    }

    let status = std::process::Command::new(&daemon_path)
        .arg("--mcp")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .context("Failed to launch aletheiad --mcp")?;

    std::process::exit(status.code().unwrap_or(1));
}
