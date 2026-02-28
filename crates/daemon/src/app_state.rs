use std::sync::{Arc, Mutex};

use anyhow::Result;
use metaygn_core::heuristics::evolver::HeuristicEvolver;
use metaygn_core::runner::ControlLoop;
use metaygn_memory::graph::GraphMemory;
use metaygn_memory::store::MemoryStore;
use metaygn_sandbox::ProcessSandbox;
use metaygn_shared::budget_tracker::SessionBudget;
use metaygn_verifiers::guard_pipeline::GuardPipeline;

use crate::forge::ForgeEngine;
use crate::profiler::fatigue::FatigueProfiler;

/// Shared application state for the daemon.
#[derive(Clone)]
pub struct AppState {
    pub memory: Arc<MemoryStore>,
    pub control_loop: Arc<ControlLoop>,
    pub guard_pipeline: Arc<GuardPipeline>,
    pub sandbox: Arc<ProcessSandbox>,
    pub fatigue: Arc<Mutex<FatigueProfiler>>,
    pub graph: Arc<GraphMemory>,
    pub evolver: Arc<Mutex<HeuristicEvolver>>,
    pub forge: Arc<Mutex<ForgeEngine>>,
    pub budget: Arc<Mutex<SessionBudget>>,
}

impl AppState {
    /// Create a new AppState backed by an in-memory SQLite database.
    /// Useful for tests.
    pub async fn new_in_memory() -> Result<Self> {
        let store = MemoryStore::open_in_memory().await?;
        let graph = GraphMemory::open_in_memory().await?;
        let sandbox = Arc::new(ProcessSandbox::with_defaults());
        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
            sandbox: sandbox.clone(),
            fatigue: Arc::new(Mutex::new(FatigueProfiler::with_defaults())),
            graph: Arc::new(graph),
            evolver: Arc::new(Mutex::new(HeuristicEvolver::new(20))),
            forge: Arc::new(Mutex::new(ForgeEngine::new(sandbox))),
            budget: Arc::new(Mutex::new(SessionBudget::new(100_000, 1.00))),
        })
    }

    /// Create a new AppState backed by a file-based SQLite database.
    pub async fn new(db_path: &str) -> Result<Self> {
        let store = MemoryStore::open(db_path).await?;
        // Graph memory shares the same directory as the main store.
        let graph_path = format!("{db_path}.graph.db");
        let graph = GraphMemory::open(&graph_path).await?;
        let sandbox = Arc::new(ProcessSandbox::with_defaults());
        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
            sandbox: sandbox.clone(),
            fatigue: Arc::new(Mutex::new(FatigueProfiler::with_defaults())),
            graph: Arc::new(graph),
            evolver: Arc::new(Mutex::new(HeuristicEvolver::new(20))),
            forge: Arc::new(Mutex::new(ForgeEngine::new(sandbox))),
            budget: Arc::new(Mutex::new(SessionBudget::new(100_000, 1.00))),
        })
    }
}
