use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};

/// Port file location: ~/.claude/aletheia/daemon.port
pub fn port_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".claude").join("aletheia").join("daemon.port"))
}

/// Read the daemon port from the port file.
/// Returns None if the file doesn't exist or can't be parsed.
pub fn read_daemon_port() -> Option<u16> {
    let path = port_file_path().ok()?;
    let contents = std::fs::read_to_string(path).ok()?;
    contents.trim().parse::<u16>().ok()
}

/// Build a reqwest client with a 2-second timeout.
pub fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .context("failed to build HTTP client")
}
