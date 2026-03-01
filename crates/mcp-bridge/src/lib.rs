use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::Value;

/// Read the daemon port from `~/.claude/aletheia/daemon.port`.
///
/// Returns `None` if the file does not exist or cannot be parsed.
pub fn read_daemon_port() -> Option<u16> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude").join("aletheia").join("daemon.port");
    let contents = std::fs::read_to_string(path).ok()?;
    contents.trim().parse::<u16>().ok()
}

/// HTTP client for communicating with the Aletheia daemon.
pub struct DaemonClient {
    base_url: String,
    client: reqwest::Client,
}

impl DaemonClient {
    /// Create a new `DaemonClient` targeting `http://127.0.0.1:{port}`.
    pub fn new(port: u16) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self {
            base_url: format!("http://127.0.0.1:{port}"),
            client,
        })
    }

    /// Send a GET request to the daemon at the given path.
    pub async fn get(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url} failed"))?;

        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .with_context(|| format!("GET {url}: failed to parse JSON response"))?;

        if !status.is_success() {
            anyhow::bail!("GET {url} returned {status}: {body}");
        }

        Ok(body)
    }

    /// Send a POST request with a JSON body to the daemon at the given path.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url} failed"))?;

        let status = resp.status();
        let resp_body: Value = resp
            .json()
            .await
            .with_context(|| format!("POST {url}: failed to parse JSON response"))?;

        if !status.is_success() {
            anyhow::bail!("POST {url} returned {status}: {resp_body}");
        }

        Ok(resp_body)
    }
}
