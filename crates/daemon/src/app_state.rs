use std::sync::Arc;

use anyhow::Result;
use metaygn_memory::store::MemoryStore;

/// Shared application state for the daemon.
#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<MemoryStore>,
}

impl AppState {
    /// Create a new AppState backed by an in-memory SQLite database.
    /// Useful for tests.
    pub async fn new_in_memory() -> Result<Self> {
        let store = MemoryStore::open_in_memory().await?;
        Ok(Self {
            memory: Arc::new(store),
        })
    }

    /// Create a new AppState backed by a file-based SQLite database.
    pub async fn new(db_path: &str) -> Result<Self> {
        let store = MemoryStore::open(db_path).await?;
        Ok(Self {
            memory: Arc::new(store),
        })
    }
}
