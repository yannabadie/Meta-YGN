use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use reqwest::Client;
use serde_json::Value;

/// Application state for the TUI dashboard.
///
/// Holds the HTTP client and the latest data fetched from the daemon.
pub struct TuiApp {
    /// Port the daemon is listening on.
    pub daemon_port: u16,
    /// Shared HTTP client (with a short timeout for TUI polling).
    pub client: Client,
    /// Latest response from `GET /health`.
    pub health: Option<Value>,
    /// Latest response from `GET /memory/stats`.
    pub memory_stats: Option<Value>,
    /// Latest response from `GET /profiler/fatigue` (placeholder until Task 20).
    pub fatigue: Option<Value>,
    /// Set to `true` when the user wants to exit.
    pub should_quit: bool,
    /// Last error message (for display in the status panel).
    pub last_error: Option<String>,
}

impl TuiApp {
    /// Create a new `TuiApp` targeting the given daemon port.
    pub fn new(daemon_port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(1500))
            .build()
            .expect("failed to build HTTP client");

        Self {
            daemon_port,
            client,
            health: None,
            memory_stats: None,
            fatigue: None,
            should_quit: false,
            last_error: None,
        }
    }

    /// Poll the daemon for the latest data.
    ///
    /// Errors are silently captured into `self.last_error` so the TUI keeps
    /// running even when the daemon is temporarily unreachable.
    pub async fn tick(&mut self) {
        let base = format!("http://127.0.0.1:{}", self.daemon_port);

        // Fetch health
        match self.client.get(format!("{base}/health")).send().await {
            Ok(resp) if resp.status().is_success() => match resp.json::<Value>().await {
                Ok(v) => {
                    self.health = Some(v);
                    self.last_error = None;
                }
                Err(e) => self.last_error = Some(format!("parse /health: {e}")),
            },
            Ok(resp) => {
                self.last_error = Some(format!("/health returned {}", resp.status()));
            }
            Err(e) => {
                self.health = None;
                self.last_error = Some(format!("daemon unreachable: {e}"));
            }
        }

        // Fetch memory stats
        match self.client.get(format!("{base}/memory/stats")).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(v) = resp.json::<Value>().await {
                    self.memory_stats = Some(v);
                }
            }
            _ => {
                // Non-critical: keep stale data
            }
        }

        // Fetch fatigue report (may 404 if endpoint not wired yet)
        match self
            .client
            .get(format!("{base}/profiler/fatigue"))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(v) = resp.json::<Value>().await {
                    self.fatigue = Some(v);
                }
            }
            _ => {
                // Silently ignore — endpoint may not exist yet
            }
        }
    }

    /// Handle a keyboard event.
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('r') => {
                // Force-refresh is handled by the next tick; we just clear
                // stale data so the user sees "loading..." briefly.
                self.health = None;
                self.memory_stats = None;
                self.fatigue = None;
            }
            _ => {}
        }
    }
}
