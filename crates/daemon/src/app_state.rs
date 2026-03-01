use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use metaygn_core::heuristics::evolver::{HeuristicEvolver, HeuristicVersion};
use metaygn_core::heuristics::fitness::{FitnessScore, SessionOutcome};
use metaygn_core::runner::ControlLoop;
use metaygn_memory::graph::GraphMemory;
use metaygn_memory::store::MemoryStore;
use metaygn_sandbox::ProcessSandbox;
use metaygn_shared::budget_tracker::SessionBudget;
use metaygn_verifiers::guard_pipeline::GuardPipeline;

use crate::forge::ForgeEngine;
use crate::profiler::fatigue::FatigueProfiler;
use crate::profiler::plasticity::PlasticityTracker;

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
    pub plasticity: Arc<Mutex<PlasticityTracker>>,
}

impl AppState {
    /// Load persisted heuristic versions and outcomes from SQLite into the evolver.
    async fn load_persisted_heuristics(memory: &MemoryStore, evolver: &mut HeuristicEvolver) {
        // Load persisted heuristic versions
        if let Ok(versions) = memory.load_heuristics().await {
            for (id, generation, parent_id, fitness_json, rw_json, ss_json, created_at) in &versions
            {
                if let (Ok(fitness), Ok(risk_weights), Ok(strategy_scores)) = (
                    serde_json::from_str::<FitnessScore>(fitness_json),
                    serde_json::from_str::<HashMap<String, f64>>(rw_json),
                    serde_json::from_str::<HashMap<String, f64>>(ss_json),
                ) {
                    let version = HeuristicVersion {
                        id: id.clone(),
                        generation: *generation,
                        parent_id: parent_id.clone(),
                        fitness,
                        risk_weights,
                        strategy_scores,
                        created_at: created_at.clone(),
                    };
                    evolver.restore_version(version);
                }
            }
            tracing::info!("Loaded {} heuristic versions from SQLite", versions.len());
        }

        // Load persisted outcomes
        if let Ok(outcomes) = memory.load_recent_outcomes(50).await {
            for outcome_json in &outcomes {
                if let Ok(outcome) = serde_json::from_value::<SessionOutcome>(outcome_json.clone())
                {
                    evolver.record_outcome(outcome);
                }
            }
            tracing::info!("Loaded {} session outcomes from SQLite", outcomes.len());
        }
    }

    /// Create a new AppState backed by an in-memory SQLite database.
    /// Useful for tests.
    pub async fn new_in_memory() -> Result<Self> {
        let store = MemoryStore::open_in_memory().await?;
        let graph = GraphMemory::open_in_memory().await?;
        let sandbox = Arc::new(ProcessSandbox::with_defaults());

        let mut evolver = HeuristicEvolver::new(20);
        Self::load_persisted_heuristics(&store, &mut evolver).await;

        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
            sandbox: sandbox.clone(),
            fatigue: Arc::new(Mutex::new(FatigueProfiler::with_defaults())),
            graph: Arc::new(graph),
            evolver: Arc::new(Mutex::new(evolver)),
            forge: Arc::new(Mutex::new(ForgeEngine::new(sandbox))),
            budget: Arc::new(Mutex::new(SessionBudget::new(100_000, 1.00))),
            plasticity: Arc::new(Mutex::new(PlasticityTracker::new())),
        })
    }

    /// Create a new AppState backed by a file-based SQLite database.
    pub async fn new(db_path: &str) -> Result<Self> {
        let store = MemoryStore::open(db_path).await?;
        // Graph memory shares the same directory as the main store.
        let graph_path = format!("{db_path}.graph.db");
        let graph = GraphMemory::open(&graph_path).await?;
        let sandbox = Arc::new(ProcessSandbox::with_defaults());

        let mut evolver = HeuristicEvolver::new(20);
        Self::load_persisted_heuristics(&store, &mut evolver).await;

        Ok(Self {
            memory: Arc::new(store),
            control_loop: Arc::new(ControlLoop::new()),
            guard_pipeline: Arc::new(GuardPipeline::new()),
            sandbox: sandbox.clone(),
            fatigue: Arc::new(Mutex::new(FatigueProfiler::with_defaults())),
            graph: Arc::new(graph),
            evolver: Arc::new(Mutex::new(evolver)),
            forge: Arc::new(Mutex::new(ForgeEngine::new(sandbox))),
            budget: Arc::new(Mutex::new(SessionBudget::new(100_000, 1.00))),
            plasticity: Arc::new(Mutex::new(PlasticityTracker::new())),
        })
    }
}
