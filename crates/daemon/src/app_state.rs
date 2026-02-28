use std::sync::Arc;

use anyhow::Result;
use metaygn_core::runner::ControlLoop;
use metaygn_memory::store::MemoryStore;
use metaygn_verifiers::guard_pipeline::GuardPipeline;

/// Shared application state for the daemon.
#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<MemoryStore>,
    pub control_loop: Arc<ControlLoop>,
    pub guard_pipeline: Arc<GuardPipeline>,
}

impl AppState {
    /// Create a new AppState backed by an in-memory SQLite database.
    /// Useful for tests.
    pub async fn new_in_memory() -> Result<Self> {
        let store = MemoryStore::open_in_memory().await?;
        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
        })
    }

    /// Create a new AppState backed by a file-based SQLite database.
    pub async fn new(db_path: &str) -> Result<Self> {
        let store = MemoryStore::open(db_path).await?;
        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
        })
    }
}
